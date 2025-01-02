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
}

impl NetStats {
    fn default() -> Self {
        Self {
            rx_packets: 0,
            rx_bytes: 0,
            rx_errors: 0,
        }
    }
}
