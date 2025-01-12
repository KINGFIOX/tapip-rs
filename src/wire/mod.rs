mod arp;
mod ethernet;
mod icmpv4;
mod tcp;

pub(crate) mod ip;
pub(crate) mod ipv4;

use crate::phy::Medium;

pub use self::ethernet::Frame as EthernetFrame;
pub use self::ethernet::{Address as EthernetAddress, EtherType as EthernetProtocol};

pub use self::icmpv4::Repr as Icmpv4Repr;
pub(crate) use self::ipv4::AddressExt as Ipv4AddressExt;
pub use self::ipv4::{
    Address as Ipv4Address, Cidr as Ipv4Cidr, Packet as Ipv4Packet, Repr as Ipv4Repr,
};

pub use self::ip::{
    Address as IpAddress, Cidr as IpCidr, Endpoint as IpEndpoint,
    ListenEndpoint as IpListenEndpoint, Protocol as IpProtocol,
};

pub use self::arp::{Operation as ArpOperation, Packet as ArpPacket, Repr as ArpRepr};

pub use self::tcp::SeqNumber as TcpSeqNumber;

/// Parsing a packet failed.
///
/// Either it is malformed, or it is not supported by smoltcp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error;
pub type Result<T> = core::result::Result<T, Error>;

mod field {
    pub type Field = ::core::ops::Range<usize>;
    pub type Rest = ::core::ops::RangeFrom<usize>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareAddress {
    Ip,
    Ethernet(EthernetAddress),
}

impl HardwareAddress {
    pub(crate) fn medium(&self) -> Medium {
        match self {
            HardwareAddress::Ip => Medium::Ip,
            HardwareAddress::Ethernet(_) => Medium::Ethernet,
        }
    }
}

impl From<EthernetAddress> for HardwareAddress {
    fn from(addr: EthernetAddress) -> Self {
        HardwareAddress::Ethernet(addr)
    }
}
