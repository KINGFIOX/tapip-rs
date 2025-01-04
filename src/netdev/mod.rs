use super::*;

pub mod veth;

#[allow(unused)]
pub trait NetDev {
    fn xmit(&mut self, buf: &[u8]) -> Result<usize>;
    fn recv(&mut self, buf: &mut [u8]) -> Result<usize>;
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
pub const PACKET_INFO: u16 = 4; // because of not setting IFF_NO_PI, which contains flags(2B) + protocol(2B)
