use super::*;

pub mod arp;
pub mod ether;
pub mod hwa;
pub mod ip;
pub mod pkbuf;

#[repr(transparent)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct be16(u16);

impl Into<u16> for be16 {
    fn into(self) -> u16 {
        u16::from_be(self.0)
    }
}

impl From<u16> for be16 {
    fn from(value: u16) -> Self {
        Self(value.to_be())
    }
}

#[repr(transparent)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct be32(u32);

impl Into<u32> for be32 {
    fn into(self) -> u32 {
        u32::from_be(self.0)
    }
}

pub type IPAddr = be32;

impl IPAddr {
    pub fn from_be(be: u32) -> Self {
        Self(be)
    }
}

impl IPAddr {
    pub fn is_multicast(&self) -> bool {
        let ip: u32 = self.0.into();
        (ip & 0xf0_00_00_00) == 0xe0_00_00_00
    }

    pub fn is_broadcast(&self) -> bool {
        let ip: u32 = self.0.into();
        (ip & 0xff_00_00_00) == 0xff_00_00_00 || (ip & 0xff_00_00_00) == 0x00_00_00_00
    }
}
