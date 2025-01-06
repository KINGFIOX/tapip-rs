use std::fmt::Debug;

use super::*;

pub mod arp;
pub mod ether;
pub mod hwa;
pub mod ip;
pub mod pkbuf;

#[repr(transparent)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct be16(u16);

impl Into<u16> for be16 {
    fn into(self) -> u16 {
        u16::from_be(self.0)
    }
}

impl be16 {
    pub fn from_le(value: u16) -> Self {
        Self(value.to_be())
    }
}

impl Debug for be16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let le: u16 = u16::from_be(self.0);
        write!(f, "{}", le)
    }
}

#[repr(transparent)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct be32(u32);

impl Into<u32> for be32 {
    fn into(self) -> u32 {
        u32::from_be(self.0)
    }
}

impl Debug for be32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let le: u32 = u32::from_be(self.0);
        write!(f, "{}", le)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct IPV4Addr(be32);

impl IPV4Addr {
    pub fn from_be(be: u32) -> Self {
        let be = be32(be);
        Self(be)
    }
}

impl Debug for IPV4Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let be = self.0;
        let le: u32 = be.into();
        write!(
            f,
            "{}.{}.{}.{}",
            (le >> 24) & 0xff,
            (le >> 16) & 0xff,
            (le >> 8) & 0xff,
            le & 0xff
        )
    }
}

impl IPV4Addr {
    pub fn is_multicast(&self) -> bool {
        let be: be32 = self.0;
        let le: u32 = be.into();
        (le & 0xf0_00_00_00) == 0xe0_00_00_00
    }

    pub fn is_broadcast(&self) -> bool {
        let be: be32 = self.0;
        let le: u32 = be.into();
        (le & 0xff_00_00_00) == 0xff_00_00_00 || (le & 0xff_00_00_00) == 0x00_00_00_00
    }
}
