use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
};

use ip::IP_ALEN;
use lazy_static::lazy_static;
use libc::{ETH_ALEN, ETH_P_ARP, ETH_P_IP};
use log::info;
use netdev::{NetDev, ETH_HRD_SZ};
use types::{
    arp::{Arp, ArpProtocol, ARP_HDR_ETHER, ARP_HRD_SZ, ARP_OP_REPLY, ARP_OP_REQUEST, ARP_TIMEOUT},
    ether::Ether,
    hwa::HardwareAddr,
    pkbuf::{PacketBuffer, PacketBufferType},
    IPAddr,
};

use super::*;

pub fn arp_in(pkbuf: Rc<RefCell<PacketBuffer>>) -> Result<()> {
    let ppacket = pkbuf.borrow();
    if ppacket.pk_type().unwrap() == PacketBufferType::Other {
        return Err(anyhow::anyhow!("this packet is not for us")).with_context(|| context!());
    }
    if ppacket.data().len() < ETH_HRD_SZ as usize + ARP_HRD_SZ {
        return Err(anyhow::anyhow!("packet too short")).with_context(|| context!());
    }
    let eth_hdr = ppacket.payload();
    let arp_hdr = eth_hdr.payload::<Arp>();
    if arp_hdr.src_hardware_addr() != eth_hdr.src() {
        return Err(anyhow::anyhow!("error sender hardware address")).with_context(|| context!());
    }
    let arp_pro = arp_hdr.protocol();
    let arp_pro: u16 = arp_pro.into();
    if arp_hdr.type_() != ARP_HDR_ETHER
        || arp_pro as i32 != ETH_P_IP
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

#[derive(PartialEq)]
enum ArpState {
    Waiting,
    Resolved,
}

struct ArpValue {
    hardware_addr: HardwareAddr,
    dev: Arc<Mutex<dyn NetDev>>,
    state: ArpState,
    ttl: u32,
}

impl ArpValue {
    fn new(dev: Arc<Mutex<dyn NetDev>>, hardware_addr: HardwareAddr) -> Self {
        Self {
            hardware_addr,
            dev,
            state: ArpState::Resolved,
            ttl: ARP_TIMEOUT,
        }
    }
}

unsafe impl Send for ArpValue {}

lazy_static! {
    static ref ARP_TABLE: Arc<Mutex<HashMap<(IPAddr, ArpProtocol), ArpValue>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

fn arp_reply(pkbuf: Rc<RefCell<PacketBuffer>>) -> Result<()> {
    // convert
    let mut ppacket = pkbuf.borrow_mut();
    let dev_handler = ppacket.dev_handler().with_context(|| context!())?;
    let eth_hdr = ppacket.payload_mut();
    let arp_hdr = eth_hdr.payload_mut::<Arp>();

    arp_hdr.set_opcode(ARP_OP_REPLY); // opcode

    // reply target
    let target_ip_addr = arp_hdr.src_ip_addr();
    arp_hdr.set_target_ip_addr(target_ip_addr);
    let target_hardware_addr = arp_hdr.src_hardware_addr();
    arp_hdr.set_target_hardware_addr(target_hardware_addr);

    // reply source
    {
        let dev = dev_handler.lock().unwrap();
        let src_hardware_addr = dev.hardware_addr();
        arp_hdr.set_src_hardware_addr(src_hardware_addr);
        let src_ip_addr = dev.ipv4_addr();
        arp_hdr.set_src_ip_addr(src_ip_addr);
    }

    drop(ppacket);

    info!("arp reply");
    let _ = PacketBuffer::send(
        pkbuf,
        target_hardware_addr,
        ETH_P_ARP as u16,
        ARP_HRD_SZ as usize,
    )
    .with_context(|| context!());
    Ok(())
}

fn arp_queue_send(pkbuf: Rc<RefCell<PacketBuffer>>) -> Result<()> {
    todo!()
}

fn arp_recv(pkbuf: Rc<RefCell<PacketBuffer>>) -> Result<()> {
    let ppacket = pkbuf.borrow();
    let eth_hdr = ppacket.payload();
    let arp_hdr = eth_hdr.payload::<Arp>();
    // filter broadcast and multicast
    if arp_hdr.target_ip_addr().is_broadcast() {
        return Err(anyhow::anyhow!("arp broadcast"));
    }
    if arp_hdr.target_ip_addr().is_multicast() {
        return Err(anyhow::anyhow!("arp multicast"));
    }
    //
    let key = (arp_hdr.src_ip_addr(), arp_hdr.protocol());
    let mut arp_table = ARP_TABLE.lock().unwrap();
    let value = arp_table.get_mut(&key);
    let src_hardware_addr = arp_hdr.src_hardware_addr();
    if let Some(value) = value {
        value.hardware_addr = src_hardware_addr; // update
        if value.state == ArpState::Waiting {
            arp_queue_send(pkbuf.clone())?;
        }
        value.state = ArpState::Resolved;
        value.ttl = ARP_TIMEOUT;
    } else {
        let value = ArpValue::new(
            ppacket.dev_handler().unwrap(), // FIXME: unwrap
            src_hardware_addr,
        );
        arp_table.insert(key, value);
    }
    let opcode = arp_hdr.opcode();
    drop(ppacket);

    if opcode == ARP_OP_REQUEST {
        arp_reply(pkbuf.clone())?;
    }

    Ok(())
}
