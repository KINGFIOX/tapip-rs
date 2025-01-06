use super::*;
use arp::arp_in;
use ip::ipv4_in;
use libc::{ETH_P_ARP, ETH_P_IP, ETH_P_RARP};
use log::info;
use netdev::ETH_HRD_SZ;
use std::{cell::RefCell, rc::Rc};
use types::{
    ether::Ether,
    pkbuf::{PacketBuffer, PacketBufferType},
};

fn eth_trans_type(pkbuf: Rc<RefCell<PacketBuffer>>) {
    // get eth header
    let mut pkbuf = pkbuf.borrow_mut();
    let payload: &[u8] = &pkbuf.payload;
    let eth_hdr = Ether::from(payload);

    // type
    let pk_type;
    if eth_hdr.dst.is_multicast() {
        if eth_hdr.dst.is_broadcast() {
            pk_type = PacketBufferType::BoardCast;
        } else {
            pk_type = PacketBufferType::Multicast;
        }
    } else {
        pk_type = PacketBufferType::Other;
    }
    pkbuf.pk_type_mut().replace(pk_type);
}

fn eth_trans_protocol(pkbuf: Rc<RefCell<PacketBuffer>>) {
    // get eth header
    let mut pkbuf = pkbuf.borrow_mut();
    let payload: &[u8] = &pkbuf.payload;
    let eth_hdr = Ether::from(payload);
    pkbuf.eth_pro_mut().replace(eth_hdr.protocol());
}

pub fn net_in(pkbuf: Rc<RefCell<PacketBuffer>>) -> Result<()> {
    if pkbuf.borrow().payload.len() < ETH_HRD_SZ as usize {
        return Err(anyhow::anyhow!("packet too short")).with_context(|| context!());
    }
    eth_trans_type(pkbuf.clone());
    eth_trans_protocol(pkbuf.clone());
    let Some(eth_pro) = pkbuf.clone().borrow().eth_pro() else {
        return Err(anyhow::anyhow!("eth_pro should not be None")).with_context(|| context!());
    };
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
        }
    }
    Ok(())
}
