use core::convert::From;
use core::fmt;

use super::{Error, Result};
use crate::phy::ChecksumCapabilities;
use crate::wire::{Ipv4Address, Ipv4AddressExt, Ipv4Cidr, Ipv4Packet, Ipv4Repr};

/// Internet protocol version.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Version {
    Ipv4,
}

impl Version {
    /// Return the version of an IP packet stored in the provided buffer.
    ///
    /// This function never returns `Ok(IpVersion::Unspecified)`; instead,
    /// unknown versions result in `Err(Error)`.
    pub const fn of_packet(data: &[u8]) -> Result<Version> {
        match data[0] >> 4 {
            4 => Ok(Version::Ipv4),
            _ => Err(Error),
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Version::Ipv4 => write!(f, "IPv4"),
        }
    }
}

enum_with_unknown! {
    /// IP datagram encapsulated protocol.
    pub enum Protocol(u8) {
        HopByHop  = 0x00,
        Icmp      = 0x01,
        Igmp      = 0x02,
        Tcp       = 0x06,
        Udp       = 0x11,
        Ipv6Route = 0x2b,
        Ipv6Frag  = 0x2c,
        IpSecEsp  = 0x32,
        IpSecAh   = 0x33,
        Icmpv6    = 0x3a,
        Ipv6NoNxt = 0x3b,
        Ipv6Opts  = 0x3c
    }
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Protocol::HopByHop => write!(f, "Hop-by-Hop"),
            Protocol::Icmp => write!(f, "ICMP"),
            Protocol::Igmp => write!(f, "IGMP"),
            Protocol::Tcp => write!(f, "TCP"),
            Protocol::Udp => write!(f, "UDP"),
            Protocol::Ipv6Route => write!(f, "IPv6-Route"),
            Protocol::Ipv6Frag => write!(f, "IPv6-Frag"),
            Protocol::IpSecEsp => write!(f, "IPsec-ESP"),
            Protocol::IpSecAh => write!(f, "IPsec-AH"),
            Protocol::Icmpv6 => write!(f, "ICMPv6"),
            Protocol::Ipv6NoNxt => write!(f, "IPv6-NoNxt"),
            Protocol::Ipv6Opts => write!(f, "IPv6-Opts"),
            Protocol::Unknown(id) => write!(f, "0x{id:02x}"),
        }
    }
}

/// An internetworking address.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Address {
    /// An IPv4 address.
    Ipv4(Ipv4Address),
}

impl Address {
    /// Create an address wrapping an IPv4 address with the given octets.
    pub const fn v4(a0: u8, a1: u8, a2: u8, a3: u8) -> Address {
        Address::Ipv4(Ipv4Address::new(a0, a1, a2, a3))
    }

    /// Return the protocol version.
    pub const fn version(&self) -> Version {
        match self {
            Address::Ipv4(_) => Version::Ipv4,
        }
    }

    /// Query whether the address is a valid unicast address.
    pub fn is_unicast(&self) -> bool {
        match self {
            Address::Ipv4(addr) => addr.x_is_unicast(),
        }
    }

    /// Query whether the address is a valid multicast address.
    pub const fn is_multicast(&self) -> bool {
        match self {
            Address::Ipv4(addr) => addr.is_multicast(),
        }
    }

    /// Query whether the address is the broadcast address.
    pub fn is_broadcast(&self) -> bool {
        match self {
            Address::Ipv4(addr) => addr.is_broadcast(),
        }
    }

    /// Query whether the address falls into the "unspecified" range.
    pub fn is_unspecified(&self) -> bool {
        match self {
            Address::Ipv4(addr) => addr.is_unspecified(),
        }
    }

    /// If `self` is a CIDR-compatible subnet mask, return `Some(prefix_len)`,
    /// where `prefix_len` is the number of leading zeroes. Return `None` otherwise.
    pub fn prefix_len(&self) -> Option<u8> {
        match self {
            Address::Ipv4(addr) => addr.prefix_len(),
        }
    }
}

impl From<Address> for ::core::net::IpAddr {
    fn from(x: Address) -> ::core::net::IpAddr {
        match x {
            Address::Ipv4(ipv4) => ::core::net::IpAddr::V4(ipv4),
        }
    }
}

impl From<Ipv4Address> for Address {
    fn from(ipv4: Ipv4Address) -> Address {
        Address::Ipv4(ipv4)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Address::Ipv4(addr) => write!(f, "{addr}"),
        }
    }
}

/// A specification of a CIDR block, containing an address and a variable-length
/// subnet masking prefix length.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Cidr {
    Ipv4(Ipv4Cidr),
}

impl Cidr {
    /// Create a CIDR block from the given address and prefix length.
    ///
    /// # Panics
    /// This function panics if the given prefix length is invalid for the given address.
    pub fn new(addr: Address, prefix_len: u8) -> Cidr {
        match addr {
            Address::Ipv4(addr) => Cidr::Ipv4(Ipv4Cidr::new(addr, prefix_len)),
        }
    }

    /// Return the IP address of this CIDR block.
    pub const fn address(&self) -> Address {
        match *self {
            Cidr::Ipv4(cidr) => Address::Ipv4(cidr.address()),
        }
    }

    /// Return the prefix length of this CIDR block.
    pub const fn prefix_len(&self) -> u8 {
        match *self {
            Cidr::Ipv4(cidr) => cidr.prefix_len(),
        }
    }

    /// Query whether the subnetwork described by this CIDR block contains
    /// the given address.
    pub fn contains_addr(&self, addr: &Address) -> bool {
        match (self, addr) {
            (Cidr::Ipv4(cidr), Address::Ipv4(addr)) => cidr.contains_addr(addr),
            #[allow(unreachable_patterns)]
            _ => false,
        }
    }

    /// Query whether the subnetwork described by this CIDR block contains
    /// the subnetwork described by the given CIDR block.
    pub fn contains_subnet(&self, subnet: &Cidr) -> bool {
        match (self, subnet) {
            (Cidr::Ipv4(cidr), Cidr::Ipv4(other)) => cidr.contains_subnet(other),
            #[allow(unreachable_patterns)]
            _ => false,
        }
    }
}

impl From<Ipv4Cidr> for Cidr {
    fn from(addr: Ipv4Cidr) -> Self {
        Cidr::Ipv4(addr)
    }
}

impl fmt::Display for Cidr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Cidr::Ipv4(cidr) => write!(f, "{cidr}"),
        }
    }
}

/// An internet endpoint address.
///
/// `Endpoint` always fully specifies both the address and the port.
///
/// See also ['ListenEndpoint'], which allows not specifying the address
/// in order to listen on a given port on any address.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Endpoint {
    pub addr: Address,
    pub port: u16,
}

impl Endpoint {
    /// Create an endpoint address from given address and port.
    pub const fn new(addr: Address, port: u16) -> Endpoint {
        Endpoint { addr, port }
    }
}

impl From<::core::net::SocketAddrV4> for Endpoint {
    fn from(x: ::core::net::SocketAddrV4) -> Endpoint {
        Endpoint {
            addr: (*x.ip()).into(),
            port: x.port(),
        }
    }
}

impl fmt::Display for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.addr, self.port)
    }
}

impl<T: Into<Address>> From<(T, u16)> for Endpoint {
    fn from((addr, port): (T, u16)) -> Endpoint {
        Endpoint {
            addr: addr.into(),
            port,
        }
    }
}

/// An internet endpoint address for listening.
///
/// In contrast with [`Endpoint`], `ListenEndpoint` allows not specifying the address,
/// in order to listen on a given port at all our addresses.
///
/// An endpoint can be constructed from a port, in which case the address is unspecified.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub struct ListenEndpoint {
    pub addr: Option<Address>,
    pub port: u16,
}

impl ListenEndpoint {
    /// Query whether the endpoint has a specified address and port.
    pub const fn is_specified(&self) -> bool {
        self.addr.is_some() && self.port != 0
    }
}

impl From<::core::net::SocketAddrV4> for ListenEndpoint {
    fn from(x: ::core::net::SocketAddrV4) -> ListenEndpoint {
        ListenEndpoint {
            addr: Some((*x.ip()).into()),
            port: x.port(),
        }
    }
}

impl fmt::Display for ListenEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(addr) = self.addr {
            write!(f, "{}:{}", addr, self.port)
        } else {
            write!(f, "*:{}", self.port)
        }
    }
}

impl From<u16> for ListenEndpoint {
    fn from(port: u16) -> ListenEndpoint {
        ListenEndpoint { addr: None, port }
    }
}

impl From<Endpoint> for ListenEndpoint {
    fn from(endpoint: Endpoint) -> ListenEndpoint {
        ListenEndpoint {
            addr: Some(endpoint.addr),
            port: endpoint.port,
        }
    }
}

impl<T: Into<Address>> From<(T, u16)> for ListenEndpoint {
    fn from((addr, port): (T, u16)) -> ListenEndpoint {
        ListenEndpoint {
            addr: Some(addr.into()),
            port,
        }
    }
}

/// An IP packet representation.
///
/// This enum abstracts the various versions of IP packets. It either contains an IPv4
/// or IPv6 concrete high-level representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Repr {
    Ipv4(Ipv4Repr),
}

impl From<Ipv4Repr> for Repr {
    fn from(repr: Ipv4Repr) -> Repr {
        Repr::Ipv4(repr)
    }
}

impl Repr {
    /// Create a new IpRepr, choosing the right IP version for the src/dst addrs.
    ///
    /// # Panics
    ///
    /// Panics if `src_addr` and `dst_addr` are different IP version.
    pub fn new(
        src_addr: Address,
        dst_addr: Address,
        next_header: Protocol,
        payload_len: usize,
        hop_limit: u8,
    ) -> Self {
        match (src_addr, dst_addr) {
            (Address::Ipv4(src_addr), Address::Ipv4(dst_addr)) => Self::Ipv4(Ipv4Repr {
                src_addr,
                dst_addr,
                next_header,
                payload_len,
                hop_limit,
            }),
            #[allow(unreachable_patterns)]
            _ => panic!("IP version mismatch: src={src_addr:?} dst={dst_addr:?}"),
        }
    }

    /// Return the protocol version.
    pub const fn version(&self) -> Version {
        match *self {
            Repr::Ipv4(_) => Version::Ipv4,
        }
    }

    /// Return the source address.
    pub const fn src_addr(&self) -> Address {
        match *self {
            Repr::Ipv4(repr) => Address::Ipv4(repr.src_addr),
        }
    }

    /// Return the destination address.
    pub const fn dst_addr(&self) -> Address {
        match *self {
            Repr::Ipv4(repr) => Address::Ipv4(repr.dst_addr),
        }
    }

    /// Return the next header (protocol).
    pub const fn next_header(&self) -> Protocol {
        match *self {
            Repr::Ipv4(repr) => repr.next_header,
        }
    }

    /// Return the payload length.
    pub const fn payload_len(&self) -> usize {
        match *self {
            Repr::Ipv4(repr) => repr.payload_len,
        }
    }

    /// Set the payload length.
    pub fn set_payload_len(&mut self, length: usize) {
        match self {
            Repr::Ipv4(Ipv4Repr { payload_len, .. }) => *payload_len = length,
        }
    }

    /// Return the TTL value.
    pub const fn hop_limit(&self) -> u8 {
        match *self {
            Repr::Ipv4(Ipv4Repr { hop_limit, .. }) => hop_limit,
        }
    }

    /// Return the length of a header that will be emitted from this high-level representation.
    pub const fn header_len(&self) -> usize {
        match *self {
            Repr::Ipv4(repr) => repr.buffer_len(),
        }
    }

    /// Emit this high-level representation into a buffer.
    pub fn emit<T: AsRef<[u8]> + AsMut<[u8]>>(
        &self,
        buffer: T,
        _checksum_caps: &ChecksumCapabilities,
    ) {
        match *self {
            Repr::Ipv4(repr) => repr.emit(&mut Ipv4Packet::new_unchecked(buffer), _checksum_caps),
        }
    }

    /// Return the total length of a packet that will be emitted from this
    /// high-level representation.
    ///
    /// This is the same as `repr.buffer_len() + repr.payload_len()`.
    pub const fn buffer_len(&self) -> usize {
        self.header_len() + self.payload_len()
    }
}

pub mod checksum {
    use byteorder::{ByteOrder, NetworkEndian};

    use super::*;

    const fn propagate_carries(word: u32) -> u16 {
        let sum = (word >> 16) + (word & 0xffff);
        ((sum >> 16) as u16) + (sum as u16)
    }

    /// Compute an RFC 1071 compliant checksum (without the final complement).
    pub fn data(mut data: &[u8]) -> u16 {
        let mut accum = 0;

        // For each 32-byte chunk...
        const CHUNK_SIZE: usize = 32;
        while data.len() >= CHUNK_SIZE {
            let mut d = &data[..CHUNK_SIZE];
            // ... take by 2 bytes and sum them.
            while d.len() >= 2 {
                accum += NetworkEndian::read_u16(d) as u32;
                d = &d[2..];
            }

            data = &data[CHUNK_SIZE..];
        }

        // Sum the rest that does not fit the last 32-byte chunk,
        // taking by 2 bytes.
        while data.len() >= 2 {
            accum += NetworkEndian::read_u16(data) as u32;
            data = &data[2..];
        }

        // Add the last remaining odd byte, if any.
        if let Some(&value) = data.first() {
            accum += (value as u32) << 8;
        }

        propagate_carries(accum)
    }

    /// Combine several RFC 1071 compliant checksums.
    pub fn combine(checksums: &[u16]) -> u16 {
        let mut accum: u32 = 0;
        for &word in checksums {
            accum += word as u32;
        }
        propagate_carries(accum)
    }

    pub fn pseudo_header_v4(
        src_addr: &Ipv4Address,
        dst_addr: &Ipv4Address,
        next_header: Protocol,
        length: u32,
    ) -> u16 {
        let mut proto_len = [0u8; 4];
        proto_len[1] = next_header.into();
        NetworkEndian::write_u16(&mut proto_len[2..4], length as u16);

        combine(&[
            data(&src_addr.octets()),
            data(&dst_addr.octets()),
            data(&proto_len[..]),
        ])
    }

    pub fn pseudo_header(
        src_addr: &Address,
        dst_addr: &Address,
        next_header: Protocol,
        length: u32,
    ) -> u16 {
        match (src_addr, dst_addr) {
            (Address::Ipv4(src_addr), Address::Ipv4(dst_addr)) => {
                pseudo_header_v4(src_addr, dst_addr, next_header, length)
            }
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }

    // We use this in pretty printer implementations.
    pub(crate) fn format_checksum(f: &mut fmt::Formatter, correct: bool) -> fmt::Result {
        if !correct {
            write!(f, " (checksum incorrect)")
        } else {
            Ok(())
        }
    }
}

use crate::wire::pretty_print::PrettyIndent;

pub fn pretty_print_ip_payload<T: Into<Repr>>(
    f: &mut fmt::Formatter,
    indent: &mut PrettyIndent,
    ip_repr: T,
    payload: &[u8],
) -> fmt::Result {
    use super::pretty_print::PrettyPrint;
    #[allow(unused)]
    use crate::wire::ip::checksum::format_checksum;
    use crate::wire::Icmpv4Packet;
    // use crate::wire::{TcpPacket, TcpRepr, UdpPacket, UdpRepr};

    #[allow(unused)]
    let checksum_caps = ChecksumCapabilities::ignored();
    let repr = ip_repr.into();
    match repr.next_header() {
        Protocol::Icmp => {
            indent.increase(f)?;
            Icmpv4Packet::<&[u8]>::pretty_print(&payload, f, indent)
        }
        // Protocol::Udp => {
        //     indent.increase(f)?;
        //     match UdpPacket::<&[u8]>::new_checked(payload) {
        //         Err(err) => write!(f, "{indent}({err})"),
        //         Ok(udp_packet) => {
        //             match UdpRepr::parse(
        //                 &udp_packet,
        //                 &repr.src_addr(),
        //                 &repr.dst_addr(),
        //                 &checksum_caps,
        //             ) {
        //                 Err(err) => write!(f, "{indent}{udp_packet} ({err})"),
        //                 Ok(udp_repr) => {
        //                     write!(
        //                         f,
        //                         "{}{} len={}",
        //                         indent,
        //                         udp_repr,
        //                         udp_packet.payload().len()
        //                     )?;
        //                     let valid =
        //                         udp_packet.verify_checksum(&repr.src_addr(), &repr.dst_addr());
        //                     format_checksum(f, valid)
        //                 }
        //             }
        //         }
        //     }
        // }
        // Protocol::Tcp => {
        //     indent.increase(f)?;
        //     match TcpPacket::<&[u8]>::new_checked(payload) {
        //         Err(err) => write!(f, "{indent}({err})"),
        //         Ok(tcp_packet) => {
        //             match TcpRepr::parse(
        //                 &tcp_packet,
        //                 &repr.src_addr(),
        //                 &repr.dst_addr(),
        //                 &checksum_caps,
        //             ) {
        //                 Err(err) => write!(f, "{indent}{tcp_packet} ({err})"),
        //                 Ok(tcp_repr) => {
        //                     write!(f, "{indent}{tcp_repr}")?;
        //                     let valid =
        //                         tcp_packet.verify_checksum(&repr.src_addr(), &repr.dst_addr());
        //                     format_checksum(f, valid)
        //                 }
        //             }
        //         }
        //     }
        // }
        _ => Ok(()),
    }
}
