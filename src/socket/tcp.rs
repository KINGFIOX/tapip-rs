use crate::storage::{Assembler, RingBuffer};
use crate::time::{Duration, Instant};
use crate::wire::{IpEndpoint, IpListenEndpoint, TcpSeqNumber};
use core::mem;

mod congestion;

const RTTE_INITIAL_RTO: u32 = 1000;
const DEFAULT_MSS: usize = 536;
const ACK_DELAY_DEFAULT: Duration = Duration::from_millis(10);

/// The state of a TCP socket, according to [RFC 793].
///
/// [RFC 793]: https://tools.ietf.org/html/rfc793
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum State {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Timer {
    Idle { keep_alive_at: Option<Instant> },
    Retransmit { expires_at: Instant },
    FastRetransmit,
    Close { expires_at: Instant },
}

impl Timer {
    fn new() -> Timer {
        Timer::Idle {
            keep_alive_at: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RttEstimator {
    /// true if we have made at least one rtt measurement.
    have_measurement: bool,
    // Using u32 instead of Duration to save space (Duration is i64)
    /// Smoothed RTT
    srtt: u32,
    /// RTT variance.
    rttvar: u32,
    /// Retransmission Time-Out
    rto: u32,
    timestamp: Option<(Instant, TcpSeqNumber)>,
    max_seq_sent: Option<TcpSeqNumber>,
    rto_count: u8,
}

impl Default for RttEstimator {
    fn default() -> Self {
        Self {
            have_measurement: false,
            srtt: 0,   // ignored, will be overwritten on first measurement.
            rttvar: 0, // ignored, will be overwritten on first measurement.
            rto: RTTE_INITIAL_RTO,
            timestamp: None,
            max_seq_sent: None,
            rto_count: 0,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Tuple {
    local: IpEndpoint,
    remote: IpEndpoint,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum AckDelayTimer {
    Idle,
    Waiting(Instant),
    Immediate,
}

/// A TCP socket ring buffer.
pub type SocketBuffer<'a> = RingBuffer<'a, u8>;

pub type TcpTimestampGenerator = fn() -> u32;

/// A Transmission Control Protocol socket.
///
/// A TCP socket may passively listen for connections or actively connect to another endpoint.
/// Note that, for listening sockets, there is no "backlog"; to be able to simultaneously
/// accept several connections, as many sockets must be allocated, or any new connection
/// attempts will be reset.
#[derive(Debug)]
pub struct Socket<'a> {
    state: State,
    timer: Timer,
    rtte: RttEstimator,
    assembler: Assembler,
    rx_buffer: SocketBuffer<'a>,
    rx_fin_received: bool,
    tx_buffer: SocketBuffer<'a>,
    /// Interval after which, if no inbound packets are received, the connection is aborted.
    timeout: Option<Duration>,
    /// Interval at which keep-alive packets will be sent.
    keep_alive: Option<Duration>,
    /// The time-to-live (IPv4) or hop limit (IPv6) value used in outgoing packets.
    hop_limit: Option<u8>,
    /// Address passed to listen(). Listen address is set when listen() is called and
    /// used every time the socket is reset back to the LISTEN state.
    listen_endpoint: IpListenEndpoint,
    /// Current 4-tuple (local and remote endpoints).
    tuple: Option<Tuple>,
    /// The sequence number corresponding to the beginning of the transmit buffer.
    /// I.e. an ACK(local_seq_no+n) packet removes n bytes from the transmit buffer.
    local_seq_no: TcpSeqNumber,
    /// The sequence number corresponding to the beginning of the receive buffer.
    /// I.e. userspace reading n bytes adds n to remote_seq_no.
    remote_seq_no: TcpSeqNumber,
    /// The last sequence number sent.
    /// I.e. in an idle socket, local_seq_no+tx_buffer.len().
    remote_last_seq: TcpSeqNumber,
    /// The last acknowledgement number sent.
    /// I.e. in an idle socket, remote_seq_no+rx_buffer.len().
    remote_last_ack: Option<TcpSeqNumber>,
    /// The last window length sent.
    remote_last_win: u16,
    /// The sending window scaling factor advertised to remotes which support RFC 1323.
    /// It is zero if the window <= 64KiB and/or the remote does not support it.
    remote_win_shift: u8,
    /// The remote window size, relative to local_seq_no
    /// I.e. we're allowed to send octets until local_seq_no+remote_win_len
    remote_win_len: usize,
    /// The receive window scaling factor for remotes which support RFC 1323, None if unsupported.
    remote_win_scale: Option<u8>,
    /// Whether or not the remote supports selective ACK as described in RFC 2018.
    remote_has_sack: bool,
    /// The maximum number of data octets that the remote side may receive.
    remote_mss: usize,
    /// The timestamp of the last packet received.
    remote_last_ts: Option<Instant>,
    /// The sequence number of the last packet received, used for sACK
    local_rx_last_seq: Option<TcpSeqNumber>,
    /// The ACK number of the last packet received.
    local_rx_last_ack: Option<TcpSeqNumber>,
    /// The number of packets received directly after
    /// each other which have the same ACK number.
    local_rx_dup_acks: u8,

    /// Duration for Delayed ACK. If None no ACKs will be delayed.
    ack_delay: Option<Duration>,
    /// Delayed ack timer. If set, packets containing exclusively
    /// ACK or window updates (ie, no data) won't be sent until expiry.
    ack_delay_timer: AckDelayTimer,

    /// Used for rate-limiting: No more challenge ACKs will be sent until this instant.
    challenge_ack_timer: Instant,

    /// Nagle's Algorithm enabled.
    nagle: bool,

    /// The congestion control algorithm.
    congestion_controller: congestion::AnyController,

    /// tsval generator - if some, tcp timestamp is enabled
    tsval_generator: Option<TcpTimestampGenerator>,

    /// 0 if not seen or timestamp not enabled
    last_remote_tsval: u32,
}

impl<'a> Socket<'a> {
    #[allow(unused_comparisons)] // small usize platforms always pass rx_capacity check
    /// Create a socket using the given buffers.
    pub fn new<T>(rx_buffer: T, tx_buffer: T) -> Socket<'a>
    where
        T: Into<SocketBuffer<'a>>,
    {
        let (rx_buffer, tx_buffer) = (rx_buffer.into(), tx_buffer.into());
        let rx_capacity = rx_buffer.capacity();

        // From RFC 1323:
        // [...] the above constraints imply that 2 * the max window size must be less
        // than 2**31 [...] Thus, the shift count must be limited to 14 (which allows
        // windows of 2**30 = 1 Gbyte).
        if rx_capacity > (1 << 30) {
            panic!("receiving buffer too large, cannot exceed 1 GiB")
        }
        let rx_cap_log2 = mem::size_of::<usize>() * 8 - rx_capacity.leading_zeros() as usize;

        Socket {
            state: State::Closed,
            timer: Timer::new(),
            rtte: RttEstimator::default(),
            assembler: Assembler::new(),
            tx_buffer,
            rx_buffer,
            rx_fin_received: false,
            timeout: None,
            keep_alive: None,
            hop_limit: None,
            listen_endpoint: IpListenEndpoint::default(),
            tuple: None,
            local_seq_no: TcpSeqNumber::default(),
            remote_seq_no: TcpSeqNumber::default(),
            remote_last_seq: TcpSeqNumber::default(),
            remote_last_ack: None,
            remote_last_win: 0,
            remote_win_len: 0,
            remote_win_shift: rx_cap_log2.saturating_sub(16) as u8,
            remote_win_scale: None,
            remote_has_sack: false,
            remote_mss: DEFAULT_MSS,
            remote_last_ts: None,
            local_rx_last_ack: None,
            local_rx_last_seq: None,
            local_rx_dup_acks: 0,
            ack_delay: Some(ACK_DELAY_DEFAULT),
            ack_delay_timer: AckDelayTimer::Idle,
            challenge_ack_timer: Instant::from_secs(0),
            nagle: true,
            tsval_generator: None,
            last_remote_tsval: 0,
            congestion_controller: congestion::AnyController::new(),
        }
    }
}
