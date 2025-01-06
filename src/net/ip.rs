use super::*;
use netdev::ETH_HRD_SZ;
use std::{cell::RefCell, rc::Rc};
use types::{
    ether::Ether,
    ip::{Ipv4, IP_HRD_SZ, IP_VERSION_4},
    pkbuf::{PacketBuffer, PacketBufferType},
};

pub const IP_ALEN: u8 = 4;

pub fn ipv4_in(pkbuf: Rc<RefCell<PacketBuffer>>) -> Result<()> {
    let ppacket = pkbuf.borrow();

    println!("ipv4_hdr: {:?}", pkbuf);

    // check packet type
    if ppacket.pk_type().unwrap() == PacketBufferType::Other {
        return Err(anyhow::anyhow!("this packet is not for us")).with_context(|| context!());
    }
    // check packet length
    if ppacket.payload.len() < ETH_HRD_SZ as usize + IP_HRD_SZ {
        return Err(anyhow::anyhow!("packet too short")).with_context(|| context!());
    }

    // get ether header
    let payload1: &[u8] = &ppacket.payload;
    let ether_hdr = Ether::from(payload1);
    let payload2 = ether_hdr.payload();
    let ipv4_hdr = Ipv4::from(payload2);

    // only version 4
    if ipv4_hdr.version() != IP_VERSION_4 {
        return Err(anyhow::anyhow!("not ipv4 packet")).with_context(|| context!());
    }

    //
    // TODO: checksum
    //

    // check packet length
    if ipv4_hdr.len() < ipv4_hdr.hlen()
        || ppacket.payload.len() < ETH_HRD_SZ as usize + ipv4_hdr.len()
    {
        return Err(anyhow::anyhow!("packet too short")).with_context(|| context!());
    }

    // meta data
    let packet_len = ppacket.payload.len();
    let ipv4_len = ipv4_hdr.len();
    drop(ppacket);

    // trim vector
    if packet_len > ipv4_len + ETH_HRD_SZ as usize {
        let len = ETH_HRD_SZ as usize + ipv4_len;
        let mut ppacket = pkbuf.borrow_mut();
        ppacket.payload = ppacket.payload[0..len].to_vec();
    }

    ip_recv_route(pkbuf).with_context(|| context!())
}

fn ip_recv_route(pkbuf: Rc<RefCell<PacketBuffer>>) -> Result<()> {
    let ppacket = pkbuf.borrow();
    let ipv4_hdr = Ipv4::from(ppacket.payload.as_ptr());
    println!("ipv4_hdr: {:?}", ipv4_hdr);
    Ok(())
}
