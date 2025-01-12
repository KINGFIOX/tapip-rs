use core::fmt;

use crate::wire::{Ipv4Address, Ipv4AddressExt, Ipv4Cidr};

/// An internetworking address.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Address {
    /// An IPv4 address.
    Ipv4(Ipv4Address),
}

impl From<Ipv4Address> for Address {
    fn from(ipv4: Ipv4Address) -> Address {
        Address::Ipv4(ipv4)
    }
}

impl Address {
    /// Query whether the address is a valid unicast address.
    pub fn is_unicast(&self) -> bool {
        match self {
            Address::Ipv4(addr) => addr.x_is_unicast(),
        }
    }

    /// Query whether the address falls into the "unspecified" range.
    pub fn is_unspecified(&self) -> bool {
        match self {
            Address::Ipv4(addr) => addr.is_unspecified(),
        }
    }

    /// Create an address wrapping an IPv4 address with the given octets.
    pub const fn v4(a0: u8, a1: u8, a2: u8, a3: u8) -> Address {
        Address::Ipv4(Ipv4Address::new(a0, a1, a2, a3))
    }
}

/// A specification of a CIDR block, containing an address and a variable-length
/// subnet masking prefix length.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Cidr {
    Ipv4(Ipv4Cidr),
}

impl Cidr {
    /// Return the IP address of this CIDR block.
    pub const fn address(&self) -> Address {
        match *self {
            Cidr::Ipv4(cidr) => Address::Ipv4(cidr.address()),
        }
    }

    /// Create a CIDR block from the given address and prefix length.
    ///
    /// # Panics
    /// This function panics if the given prefix length is invalid for the given address.
    pub fn new(addr: Address, prefix_len: u8) -> Cidr {
        match addr {
            Address::Ipv4(addr) => Cidr::Ipv4(Ipv4Cidr::new(addr, prefix_len)),
        }
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Address::Ipv4(addr) => write!(f, "{addr}"),
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
