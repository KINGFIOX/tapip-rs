use std::{
    collections::HashMap,
    mem,
    sync::{Arc, Mutex},
};

use ipv4::IP_ALEN;
use lazy_static::lazy_static;
use libc::{ETH_ALEN, ETH_P_ARP, ETH_P_IP};
use log::{info, trace};
use netdev::{NetDev, ETH_HRD_SZ};
use types::{
    arp::{Arp, ArpProtocol, ARP_HDR_ETHER, ARP_HRD_SZ, ARP_OP_REPLY, ARP_OP_REQUEST, ARP_TIMEOUT},
    hwa::HardwareAddr,
    pkbuf::{PacketBuffer, PacketBufferType},
    Ipv4Addr,
};

use super::*;

pub fn arp_in(pkbuf: Box<PacketBuffer>) -> Result<()> {
    if pkbuf.pk_type().unwrap() == PacketBufferType::Other {
        return Err(anyhow::anyhow!("this packet is not for us")).with_context(|| context!());
    }
    if pkbuf.data().len() < ETH_HRD_SZ as usize + ARP_HRD_SZ {
        return Err(anyhow::anyhow!("packet too short")).with_context(|| context!());
    }
    let eth_hdr = pkbuf.payload();
    let arp_hdr = eth_hdr.payload::<Arp>();
    if arp_hdr.source_hardware_addr() != eth_hdr.src() {
        return Err(anyhow::anyhow!("error sender hardware address")).with_context(|| context!());
    }
    let arp_pro = arp_hdr.protocol_type();
    let arp_pro: u16 = arp_pro.into();
    if arp_hdr.hardware_type() != ARP_HDR_ETHER
        || arp_pro as i32 != ETH_P_IP
        || arp_hdr.hardware_len() as i32 != ETH_ALEN
        || arp_hdr.protocol_len() != IP_ALEN
    {
        return Err(anyhow::anyhow!("unsupported L2/L3 protocol")).with_context(|| context!());
    }
    if arp_hdr.operation() != ARP_OP_REQUEST && arp_hdr.operation() != ARP_OP_REPLY {
        return Err(anyhow::anyhow!("unsupported ARP opcode")).with_context(|| context!());
    }
    arp_recv(pkbuf)
}

#[derive(PartialEq, Debug)]
enum ArpState {
    Waiting,
    Resolved,
}

#[derive(Debug)]
struct ArpValue {
    waiters: Vec<Box<PacketBuffer>>,
    hardware_addr: HardwareAddr,
    state: ArpState,
    ttl: u32,
}

impl ArpValue {
    #[allow(unused)]
    fn new(dev: Arc<Mutex<dyn NetDev>>, hardware_addr: HardwareAddr) -> Self {
        Self {
            waiters: Vec::new(),
            hardware_addr,
            state: ArpState::Resolved,
            ttl: ARP_TIMEOUT,
        }
    }
}

unsafe impl Send for ArpValue {}

lazy_static! {
    static ref ARP_TABLE: Arc<Mutex<HashMap<(Ipv4Addr, ArpProtocol), ArpValue>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

fn arp_reply(mut pkbuf: Box<PacketBuffer>) -> Result<()> {
    // convert
    let dev_handler = pkbuf.dev_handler().with_context(|| context!())?;
    let eth_hdr = pkbuf.payload_mut();
    let arp_hdr = eth_hdr.payload_mut::<Arp>();

    arp_hdr.set_operation(ARP_OP_REPLY); // opcode

    // reply target
    let target_ip_addr = arp_hdr.source_ipv4_addr();
    arp_hdr.set_target_ipv4_addr(target_ip_addr);
    let target_hardware_addr = arp_hdr.source_hardware_addr();
    arp_hdr.set_target_hardware_addr(target_hardware_addr);

    // reply source
    {
        let dev = dev_handler.lock().unwrap();
        let src_hardware_addr = dev.hardware_addr();
        arp_hdr.set_source_hardware_addr(src_hardware_addr);
        let src_ip_addr = dev.ipv4_addr();
        arp_hdr.set_source_ipv4_addr(src_ip_addr);
    }

    info!("arp reply");
    let _ = PacketBuffer::send(
        &mut pkbuf,
        target_hardware_addr,
        ETH_P_ARP as u16,
        ARP_HRD_SZ as usize,
    )
    .with_context(|| context!());
    Ok(())
}

fn arp_queue_send(value: &mut ArpValue) -> Result<()> {
    let pkbufs = mem::replace(&mut value.waiters, Vec::new());
    for mut pkbuf in pkbufs {
        let _ = PacketBuffer::send(
            &mut pkbuf,
            value.hardware_addr,
            ETH_P_ARP as u16,
            ARP_HRD_SZ as usize,
        )
        .with_context(|| context!());
    }
    info!("arp queue send");
    Ok(())
}

fn arp_recv(pkbuf: Box<PacketBuffer>) -> Result<()> {
    let eth_hdr = pkbuf.payload();
    let arp_hdr = eth_hdr.payload::<Arp>();
    // filter broadcast and multicast
    if arp_hdr.target_ipv4_addr().is_broadcast() {
        return Err(anyhow::anyhow!("arp broadcast"));
    }
    if arp_hdr.target_ipv4_addr().is_multicast() {
        return Err(anyhow::anyhow!("arp multicast"));
    }

    let key = (arp_hdr.source_ipv4_addr(), arp_hdr.protocol_type());
    let mut arp_table = ARP_TABLE.lock().unwrap();
    let value = arp_table.get_mut(&key);
    let src_hardware_addr = arp_hdr.source_hardware_addr();
    let dev = pkbuf.dev_handler().unwrap();
    let opcode = arp_hdr.operation();

    if let Some(value) = value {
        value.hardware_addr = src_hardware_addr; // update
        if value.state == ArpState::Waiting {
            arp_queue_send(value)?;
        }
        value.state = ArpState::Resolved;
        value.ttl = ARP_TIMEOUT;
    } else if opcode == ARP_OP_REQUEST {
        let value = ArpValue::new(dev, src_hardware_addr);
        arp_table.insert(key, value);
    }

    trace!("arp table: {:?}", arp_table);

    if opcode == ARP_OP_REQUEST {
        arp_reply(pkbuf)?;
    }

    Ok(())
}
