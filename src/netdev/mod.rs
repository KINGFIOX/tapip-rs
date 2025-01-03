use super::*;

pub mod veth;

#[allow(unused)]
pub trait NetDev {
    fn xmit(&mut self, buf: &[u8]) -> Result<usize>;
    fn recv(&mut self, buf: &mut [u8]) -> Result<usize>;
}

#[derive(Debug)]
pub struct NetStats {
    rx_packets: u64,
    rx_bytes: u64,
    rx_errors: u64,
    tx_packets: u64,
    tx_errors: u64,
    tx_bytes: u64,
}

impl NetStats {
    fn default() -> Self {
        Self {
            rx_packets: 0,
            rx_bytes: 0,
            rx_errors: 0,
            tx_packets: 0,
            tx_errors: 0,
            tx_bytes: 0,
        }
    }
}

pub const MTU: u16 = 1500;
pub const ETH_HRD_SZ: u16 = 14;
pub const PACKET_INFO: u16 = 4; // because of not setting IFF_NO_PI, which contains flags(2B) + protocol(2B)
