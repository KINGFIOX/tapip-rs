use super::*;
use netdev::ETH_HRD_SZ;
use types::{
    ipv4::{Ipv4Header, IP_HRD_SZ, IP_VERSION_4},
    pkbuf::{PacketBuffer, PacketBufferType},
};

pub const IP_ALEN: u8 = 4;

pub fn ipv4_in(mut pkbuf: Box<PacketBuffer>) -> Result<()> {
    // check packet type
    if pkbuf.pk_type().unwrap() == PacketBufferType::Other {
        return Err(anyhow::anyhow!("this packet is not for us")).with_context(|| context!());
    }
    // check packet length
    if pkbuf.data().len() < ETH_HRD_SZ as usize + IP_HRD_SZ {
        return Err(anyhow::anyhow!("packet too short")).with_context(|| context!());
    }

    // get ether header
    let ether_hdr = pkbuf.payload();
    let ipv4_hdr = ether_hdr.payload::<Ipv4Header>();

    // only version 4
    if ipv4_hdr.version() != IP_VERSION_4 {
        return Err(anyhow::anyhow!("not ipv4 packet {:?}", ipv4_hdr)).with_context(|| context!());
    }

    // checksum
    if !ipv4_hdr.verify_checksum() {
        return Err(anyhow::anyhow!("checksum error")).with_context(|| context!());
    }

    // check packet length
    if ipv4_hdr.total_len() < ipv4_hdr.header_len()
        || pkbuf.data().len() < ETH_HRD_SZ as usize + ipv4_hdr.total_len()
    {
        return Err(anyhow::anyhow!("packet too short")).with_context(|| context!());
    }

    // meta data
    let packet_len = pkbuf.data().len();
    let ipv4_len = ipv4_hdr.total_len();

    // trim vector
    if packet_len > ipv4_len + ETH_HRD_SZ as usize {
        let len = ETH_HRD_SZ as usize + ipv4_len;
        let ppacket = &mut pkbuf;
        *ppacket.data_mut() = ppacket.data()[0..len].to_vec();
    }

    ip_recv_route(pkbuf).with_context(|| context!())
}

#[allow(unused)]
fn ip_recv_route(pkbuf: Box<PacketBuffer>) -> Result<()> {
    todo!()
}
