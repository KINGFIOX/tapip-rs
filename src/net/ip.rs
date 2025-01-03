use etherparse::Ipv4HeaderSlice;

use super::*;

fn as_ipv4(payload: &[u8], offset: u16) -> Result<Ipv4HeaderSlice> {
    let (_, payload) = as_ether(payload, offset)?;
    let header = Ipv4HeaderSlice::from_slice(payload).with_context(|| context!())?;
    Ok(header)
}

pub fn ipv4_in(payload: &[u8]) -> Result<()> {
    let header = as_ipv4(payload, PACKET_INFO).with_context(|| context!())?;
    Ok(())
}
