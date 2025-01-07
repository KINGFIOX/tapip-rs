use super::*;
use lazy_static::lazy_static;
use netdev::veth::veth_ip_addr;
use std::sync::{Arc, Mutex};
use types::{ipv4::Ipv4Header, pkbuf::PacketBuffer, Ipv4Addr, Ipv4Mask};

lazy_static! {
    static ref ROUTE_TABLE: Arc<Mutex<Vec<RouteEntry>>> = {
        let mut route_table = Vec::new();
        route_add_(
            RouteEntry::new(
                Ipv4Addr::new(10, 0, 0, 2),
                Ipv4Mask::prefix_new(0),
                veth_ip_addr(),
                RouteEntryType::Localhost,
            ),
            &mut route_table,
        );
        Arc::new(Mutex::new(route_table))
    };
}

#[allow(unused)]
#[derive(Clone, Copy, PartialEq)]
pub enum RouteEntryType {
    None,
    Default,
    Localhost,
}

#[derive(Clone)]
pub struct RouteEntry {
    ip_addr: Ipv4Addr,
    netmask: Ipv4Mask,
    gateway: Ipv4Addr,
    entry_type: RouteEntryType,
}

unsafe impl Send for RouteEntry {}

impl RouteEntry {
    fn new(
        ip_addr: Ipv4Addr,
        netmask: Ipv4Mask,
        gateway: Ipv4Addr,
        entry_type: RouteEntryType,
    ) -> Self {
        Self {
            ip_addr,
            netmask,
            gateway,
            entry_type,
        }
    }
}

#[allow(unused)]
impl RouteEntry {
    pub fn ip_addr(&self) -> Ipv4Addr {
        self.ip_addr
    }
    pub fn netmask(&self) -> Ipv4Mask {
        self.netmask
    }
    pub fn gateway(&self) -> Ipv4Addr {
        self.gateway
    }
    pub fn entry_type(&self) -> RouteEntryType {
        self.entry_type
    }
}

fn route_add_(entry: RouteEntry, table: &mut Vec<RouteEntry>) {
    for (index, existing_entry) in table.iter().enumerate() {
        if entry.netmask >= existing_entry.netmask {
            table.insert(index + 1, entry);
            return;
        }
    }
    table.push(entry);
}

#[allow(unused)]
fn route_add(entry: RouteEntry) {
    let mut route_table = ROUTE_TABLE.lock().unwrap();
    route_add_(entry, &mut route_table);
}

fn rt_lookup(ip_addr: Ipv4Addr) -> Option<RouteEntry> {
    let route_table = ROUTE_TABLE.lock().unwrap();
    for entry in route_table.iter() {
        let mask = entry.netmask;
        let net = entry.ip_addr;
        if ip_addr.mask(&mask) == net.mask(&mask) {
            return Some(entry.clone());
        }
    }
    None
}

#[allow(unused)]
pub fn rt_input(pkbuf: &PacketBuffer) -> Result<RouteEntry> {
    let eth_hdr = pkbuf.payload();
    let ip_hdr = eth_hdr.payload::<Ipv4Header>();
    if let Some(entry) = rt_lookup(ip_hdr.dst_addr()) {
        Ok(entry)
    } else {
        todo!()
    }
}
