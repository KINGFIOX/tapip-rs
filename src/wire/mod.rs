mod ethernet;

pub(crate) mod ip;
pub(crate) mod ipv4;

mod tcp;

use crate::phy::Medium;

pub use self::ethernet::Address as EthernetAddress;
pub use self::ethernet::Frame as EthernetFrame;

pub(crate) use self::ipv4::AddressExt as Ipv4AddressExt;
pub use self::ipv4::{Address as Ipv4Address, Cidr as Ipv4Cidr};

pub use self::ip::{
    Address as IpAddress, Cidr as IpCidr, Endpoint as IpEndpoint,
    ListenEndpoint as IpListenEndpoint,
};

pub use self::tcp::SeqNumber as TcpSeqNumber;

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
