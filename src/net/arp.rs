use std::{cell::RefCell, rc::Rc};

use ip::IP_ALEN;
use libc::{ETH_ALEN, ETH_P_IP};
use log::trace;
use netdev::ETH_HRD_SZ;
use types::{
    arp::{Arp, ARP_HDR_ETHER, ARP_HRD_SZ, ARP_OP_REPLY, ARP_OP_REQUEST},
    ether::Ether,
    pkbuf::{PacketBuffer, PacketBufferType},
};

use super::*;

pub fn arp_in(pkbuf: Rc<RefCell<PacketBuffer>>) -> Result<()> {
    let ppacket = pkbuf.borrow();
    let payload: &[u8] = &ppacket.payload;
    if ppacket.pk_type().unwrap() == PacketBufferType::Other {
        return Err(anyhow::anyhow!("this packet is not for us")).with_context(|| context!());
    }
    if ppacket.payload.len() < ETH_HRD_SZ as usize + ARP_HRD_SZ {
        return Err(anyhow::anyhow!("packet too short")).with_context(|| context!());
    }
    let eth_hdr = Ether::from(payload);
    let arp_hdr = Arp::from(payload);
    if arp_hdr.src_hardware_addr() != eth_hdr.src() {
        return Err(anyhow::anyhow!("not for us")).with_context(|| context!());
    }
    if arp_hdr.type_() != ARP_HDR_ETHER
        || arp_hdr.protocol() as i32 != ETH_P_IP
        || arp_hdr.hdr_len() as i32 != ETH_ALEN
        || arp_hdr.pro_len() != IP_ALEN
    {
        return Err(anyhow::anyhow!("unsupported L2/L3 protocol")).with_context(|| context!());
    }
    if arp_hdr.opcode() != ARP_OP_REQUEST && arp_hdr.opcode() != ARP_OP_REPLY {
        return Err(anyhow::anyhow!("unsupported ARP opcode")).with_context(|| context!());
    }
    drop(ppacket);
    arp_recv(pkbuf)
}

fn arp_recv(pkbuf: Rc<RefCell<PacketBuffer>>) -> Result<()> {
    trace!("arp_recv");
    Ok(())
}
