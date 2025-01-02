use anyhow::Result;

use super::*;

mod veth;

#[allow(unused)]
pub trait NetDev {
    fn xmit(&mut self, buf: &[u8]) -> Result<usize>;
    fn recv(&mut self, buf: &mut [u8]) -> Result<usize>;
}

pub struct NetStats {
    rx_packets: u64,
    rx_bytes: u64,
    rx_errors: u64,
}
