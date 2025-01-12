use crate::phy::PacketMeta;
use crate::wire::{IpAddress, IpEndpoint, IpListenEndpoint};

/// A UDP packet ring buffer.
pub type PacketBuffer<'a> = crate::storage::PacketBuffer<'a, UdpMetadata>;

/// A UDP packet metadata.
pub type PacketMetadata = crate::storage::PacketMetadata<UdpMetadata>;

/// Metadata for a sent or received UDP packet.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct UdpMetadata {
    /// The IP endpoint from which an incoming datagram was received, or to which an outgoing
    /// datagram will be sent.
    pub endpoint: IpEndpoint,
    /// The IP address to which an incoming datagram was sent, or from which an outgoing datagram
    /// will be sent. Incoming datagrams always have this set. On outgoing datagrams, if it is not
    /// set, and the socket is not bound to a single address anyway, a suitable address will be
    /// determined using the algorithms of RFC 6724 (candidate source address selection) or some
    /// heuristic (for IPv4).
    pub local_address: Option<IpAddress>,
    pub meta: PacketMeta,
}

/// A User Datagram Protocol socket.
///
/// A UDP socket is bound to a specific endpoint, and owns transmit and receive
/// packet buffers.
#[derive(Debug)]
pub struct Socket<'a> {
    endpoint: IpListenEndpoint,
    rx_buffer: PacketBuffer<'a>,
    tx_buffer: PacketBuffer<'a>,
    /// The time-to-live (IPv4) or hop limit (IPv6) value used in outgoing packets.
    hop_limit: Option<u8>,
}

impl<'a> Socket<'a> {
    /// Create an UDP socket with the given buffers.
    pub fn new(rx_buffer: PacketBuffer<'a>, tx_buffer: PacketBuffer<'a>) -> Socket<'a> {
        Socket {
            endpoint: IpListenEndpoint::default(),
            rx_buffer,
            tx_buffer,
            hop_limit: None,
        }
    }
}
