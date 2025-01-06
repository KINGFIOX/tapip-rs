use super::*;
use netdev::ETH_HRD_SZ;
use std::{cell::RefCell, rc::Rc};
use types::{
    ip::{IPV4Hdr, IP_HRD_SZ, IP_VERSION_4},
    pkbuf::{PacketBuffer, PacketBufferType},
};

pub const IP_ALEN: u8 = 4;

fn sum(data: &[u8], mut origsum: u32) -> u32 {
    let mut size = data.len();
    let mut i = 0;
    while size > 1 {
        let word = u16::from_le_bytes([data[i], data[i + 1]]);
        origsum += word as u32;
        i += 2;
        size -= 2;
    }
    if size != 0 {
        let word = u16::from_le_bytes([data[i], 0]);
        origsum += word as u32;
    }
    origsum
}

fn checksum(data: &[u8], origsum: u32) -> u16 {
    let mut origsum = sum(data, origsum);
    origsum = (origsum & 0xffff) + (origsum >> 16);
    origsum = (origsum & 0xffff) + (origsum >> 16);
    (!origsum & 0xffff) as u16
}

fn ipv4_chksum(data: &[u8]) -> u16 {
    checksum(data, 0)
}

pub fn ipv4_in(pkbuf: Rc<RefCell<PacketBuffer>>) -> Result<()> {
    let ppacket = pkbuf.borrow();

    // check packet type
    if ppacket.pk_type().unwrap() == PacketBufferType::Other {
        return Err(anyhow::anyhow!("this packet is not for us")).with_context(|| context!());
    }
    // check packet length
    if ppacket.data().len() < ETH_HRD_SZ as usize + IP_HRD_SZ {
        return Err(anyhow::anyhow!("packet too short")).with_context(|| context!());
    }

    // get ether header
    let ether_hdr = ppacket.payload();
    let ipv4_hdr = ether_hdr.payload::<IPV4Hdr>();

    // only version 4
    if ipv4_hdr.version() != IP_VERSION_4 {
        return Err(anyhow::anyhow!("not ipv4 packet")).with_context(|| context!());
    }

    // // checksum
    // if ipv4_chksum(ppacket.data()) != 0 {
    //     return Err(anyhow::anyhow!("checksum error")).with_context(|| context!());
    // }

    // check packet length
    if ipv4_hdr.len() < ipv4_hdr.hlen()
        || ppacket.data().len() < ETH_HRD_SZ as usize + ipv4_hdr.len()
    {
        return Err(anyhow::anyhow!("packet too short")).with_context(|| context!());
    }

    // meta data
    let packet_len = ppacket.data().len();
    let ipv4_len = ipv4_hdr.len();
    drop(ppacket);

    // trim vector
    if packet_len > ipv4_len + ETH_HRD_SZ as usize {
        let len = ETH_HRD_SZ as usize + ipv4_len;
        let mut ppacket = pkbuf.borrow_mut();
        *ppacket.data_mut() = ppacket.data()[0..len].to_vec();
    }

    ip_recv_route(pkbuf).with_context(|| context!())
}

#[allow(unused)]
fn ip_recv_route(pkbuf: Rc<RefCell<PacketBuffer>>) -> Result<()> {
    todo!()
}
