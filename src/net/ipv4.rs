use super::*;
use netdev::ETH_HRD_SZ;
use route::{rt_input, RouteEntryType};
use types::{
    ipv4::{Ipv4Header, Ipv4Protocol, IP_HRD_SZ, IP_VERSION_4},
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

fn ip_recv_route(mut pkbuf: Box<PacketBuffer>) -> Result<()> {
    let rt_entry = rt_input(&mut pkbuf).with_context(|| context!())?;
    if rt_entry.entry_type() == RouteEntryType::Localhost {
        ip_recv_local(pkbuf).with_context(|| context!())?;
    } else {
        ip_forward(pkbuf).with_context(|| context!())?;
    }
    Ok(())
}

const IP_FRAG_OFF: u16 = 0x1fff; // fragment offset mask
const IP_FRAG_MF: u16 = 0x2000; // more fragment flags
const IP_FRAG_DF: u16 = 0x4000; // dont fragment flags

fn ip_recv_local(mut pkbuf: Box<PacketBuffer>) -> Result<()> {
    let mut ether_hdr = pkbuf.payload();
    let mut ipv4_hdr = ether_hdr.payload::<Ipv4Header>();
    if ipv4_hdr.frag_off() & (IP_FRAG_OFF | IP_FRAG_MF) != 0 {
        if ipv4_hdr.frag_off() & IP_FRAG_DF != 0 {
            return Err(anyhow::anyhow!("error fragment")).with_context(|| context!());
        }
        pkbuf = ip_reass(pkbuf).with_context(|| context!())?;
        ether_hdr = pkbuf.payload();
        ipv4_hdr = ether_hdr.payload::<Ipv4Header>();
    }
    let protocol = ipv4_hdr.protocol();
    raw_in(&mut pkbuf).with_context(|| context!())?;
    match protocol {
        Ipv4Protocol::ICMP => icmp_in(pkbuf).with_context(|| context!())?,
        Ipv4Protocol::TCP => tcp_in(pkbuf).with_context(|| context!())?,
        Ipv4Protocol::UDP => udp_in(pkbuf).with_context(|| context!())?,
        Ipv4Protocol::UNKNOWN => todo!(),
    }
    Ok(())
}

fn raw_in(_pkbuf: &mut PacketBuffer) -> Result<()> {
    todo!()
}

fn icmp_in(mut _pkbuf: Box<PacketBuffer>) -> Result<()> {
    todo!()
}

fn tcp_in(mut _pkbuf: Box<PacketBuffer>) -> Result<()> {
    todo!()
}

fn udp_in(mut _pkbuf: Box<PacketBuffer>) -> Result<()> {
    todo!()
}

/// reassemble fragmented packet
fn ip_reass(mut _pkbuf: Box<PacketBuffer>) -> Result<Box<PacketBuffer>> {
    todo!()
}

fn ip_forward(mut _pkbuf: Box<PacketBuffer>) -> Result<()> {
    todo!()
}
