use super::*;

mod ip;

use etherparse::{EtherType, Ethernet2Header};
use netdev::PACKET_INFO;

fn as_ether(payload: &[u8], offset: u16) -> Result<(Ethernet2Header, &[u8])> {
    let offset = offset as usize;
    Ethernet2Header::from_slice(&payload[offset..]).with_context(|| context!())
}

pub fn net_in(payload: &[u8]) -> Result<()> {
    let (header, payload) = as_ether(payload, PACKET_INFO).with_context(|| context!())?;
    match header.ether_type {
        EtherType::IPV4 => ip::ipv4_in(payload).with_context(|| context!())?,
        EtherType::ARP => todo!(),
        _ => todo!(),
    }
    Ok(())
}
