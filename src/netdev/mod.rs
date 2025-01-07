use super::*;
use anyhow::Result;
use types::{hwa::HardwareAddr, Ipv4Addr};

pub mod veth;

#[allow(unused)]
pub trait NetDev {
    fn xmit(&mut self, buf: &[u8]) -> Result<usize>;
    fn recv(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn hardware_addr(&self) -> HardwareAddr;
    fn ipv4_addr(&self) -> Ipv4Addr;
}

#[derive(Debug)]
struct Statistics {
    packets: u64,
    errors: u64,
    bytes: u64,
}

#[derive(Debug)]
pub struct NetStats {
    rx: Statistics,
    tx: Statistics,
}

impl NetStats {
    fn default() -> Self {
        Self {
            rx: Statistics {
                packets: 0,
                errors: 0,
                bytes: 0,
            },
            tx: Statistics {
                packets: 0,
                errors: 0,
                bytes: 0,
            },
        }
    }
}

pub const MTU: u16 = 1500;
pub const ETH_HRD_SZ: u16 = 14;
