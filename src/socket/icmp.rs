use crate::wire::{IpAddress, IpListenEndpoint};

/// An ICMP packet ring buffer.
pub type PacketBuffer<'a> = crate::storage::PacketBuffer<'a, IpAddress>;

/// An ICMP packet metadata.
pub type PacketMetadata = crate::storage::PacketMetadata<IpAddress>;

/// Type of endpoint to bind the ICMP socket to. See [IcmpSocket::bind] for
/// more details.
///
/// [IcmpSocket::bind]: struct.IcmpSocket.html#method.bind
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Endpoint {
    #[default]
    Unspecified,
    Ident(u16),
    Udp(IpListenEndpoint),
}

/// A ICMP socket
///
/// An ICMP socket is bound to a specific [IcmpEndpoint] which may
/// be a specific UDP port to listen for ICMP error messages related
/// to the port or a specific ICMP identifier value. See [bind] for
/// more details.
///
/// [IcmpEndpoint]: enum.IcmpEndpoint.html
/// [bind]: #method.bind
#[derive(Debug)]
pub struct Socket<'a> {
    rx_buffer: PacketBuffer<'a>,
    tx_buffer: PacketBuffer<'a>,
    /// The endpoint this socket is communicating with
    endpoint: Endpoint,
    /// The time-to-live (IPv4) or hop limit (IPv6) value used in outgoing packets.
    hop_limit: Option<u8>,
}

impl<'a> Socket<'a> {
    /// Create an ICMP socket with the given buffers.
    pub fn new(rx_buffer: PacketBuffer<'a>, tx_buffer: PacketBuffer<'a>) -> Socket<'a> {
        Socket {
            rx_buffer,
            tx_buffer,
            endpoint: Default::default(),
            hop_limit: None,
        }
    }
}
