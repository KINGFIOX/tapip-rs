use super::*;
use lazy_static::lazy_static;
use netdev::{
    veth::{veth_handler, veth_ip_addr},
    NetDev,
};
use std::sync::{Arc, Mutex};
use types::{ipv4::Ipv4Header, pkbuf::PacketBuffer, Ipv4Addr, Ipv4Mask};

lazy_static! {
    static ref ROUTE_TABLE: Arc<Mutex<Vec<RouteEntry>>> = {
        let mut route_table = Vec::new();
        // default route
        route_table.push(RouteEntry::new(
            Ipv4Addr::new(0, 0, 0, 0),
            Ipv4Mask::prefix_new(0),
            veth_ip_addr(),
            RouteEntryType::Default,
            veth_handler(),
        ));
        Arc::new(Mutex::new(route_table))
    };
}

#[allow(unused)]
#[derive(Clone)]
enum RouteEntryType {
    None,
    Default,
    Localhost,
}

#[allow(unused)]
#[derive(Clone)]
struct RouteEntry {
    ip_addr: Ipv4Addr,
    netmask: Ipv4Mask,
    gateway: Ipv4Addr,
    entry_type: RouteEntryType,
    dev_handler: Arc<Mutex<dyn NetDev>>,
}

unsafe impl Send for RouteEntry {}

impl RouteEntry {
    pub fn new(
        ip_addr: Ipv4Addr,
        netmask: Ipv4Mask,
        gateway: Ipv4Addr,
        entry_type: RouteEntryType,
        dev_handler: Arc<Mutex<dyn NetDev>>,
    ) -> Self {
        Self {
            ip_addr,
            netmask,
            gateway,
            entry_type,
            dev_handler,
        }
    }
}

#[allow(unused)]
fn route_add(entry: RouteEntry) {
    let mut route_table = ROUTE_TABLE.lock().unwrap();
    for (index, existing_entry) in route_table.iter().enumerate() {
        if entry.netmask >= existing_entry.netmask {
            route_table.insert(index + 1, entry);
            return;
        }
    }
    route_table.push(entry);
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
pub fn rt_input(pkbuf: &PacketBuffer) -> Result<()> {
    let eth_hdr = pkbuf.payload();
    let ip_hdr = eth_hdr.payload::<Ipv4Header>();
    if let Some(entry) = rt_lookup(ip_hdr.dst_addr()) {
    } else {
    }
    todo!()
}
