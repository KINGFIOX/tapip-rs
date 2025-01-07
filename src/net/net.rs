use super::*;
use arp::arp_in;
use ipv4::ipv4_in;
use libc::{ETH_P_ARP, ETH_P_IP, ETH_P_RARP};
use log::info;
use netdev::ETH_HRD_SZ;
use types::pkbuf::{PacketBuffer, PacketBufferType};

fn eth_trans_type(pkbuf: &mut PacketBuffer) {
    // get eth header
    let eth_hdr = pkbuf.payload();

    // type
    let pk_type;
    if eth_hdr.dst().is_multicast() {
        if eth_hdr.dst().is_broadcast() {
            pk_type = PacketBufferType::BoardCast;
        } else {
            pk_type = PacketBufferType::Multicast;
        }
    } else {
        pk_type = PacketBufferType::Other;
    }
    pkbuf.pk_type_mut().replace(pk_type);
}

fn eth_trans_protocol(pkbuf: &mut PacketBuffer) -> u16 {
    // get eth header
    let eth_hdr = pkbuf.payload();
    eth_hdr.protocol()
}

pub fn net_in(mut pkbuf: Box<PacketBuffer>) -> Result<()> {
    if pkbuf.data().len() < ETH_HRD_SZ as usize {
        return Err(anyhow::anyhow!("packet too short")).with_context(|| context!());
    }
    eth_trans_type(&mut pkbuf);
    let eth_pro = eth_trans_protocol(&mut pkbuf);
    match eth_pro as i32 {
        ETH_P_RARP => {
            info!("eth_pro is RARP");
        }
        ETH_P_ARP => {
            info!("eth_pro is ARP");
            arp_in(pkbuf)?;
        }
        ETH_P_IP => {
            info!("eth_pro is IP");
            ipv4_in(pkbuf)?;
        }
        _ => {
            info!("eth_pro is other");
            // trace!("packet: {:?}", pkbuf.borrow().data());
        }
    }
    Ok(())
}
