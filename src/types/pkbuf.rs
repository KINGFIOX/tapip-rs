use crate::netdev::NetDev;

use anyhow::Result;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

lazy_static! {
    pub static ref ALLOC_PKGS: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
}

/// 一定得有一个 cache 用来存放 packet 的
/// - receive packet 是异步的, 一个后台线程轮询
/// - tcp 会有流量控制, 因此也一定要有一个 buffer
pub struct PacketBuffer {
    pub dev_handler: Option<Arc<Mutex<dyn NetDev>>>,
    pub payload: Vec<u8>,
}

impl PacketBuffer {
    pub fn new(reserved: u16) -> Result<Self> {
        let mut allo = ALLOC_PKGS.lock().unwrap();
        *allo += 1;
        let pkbuf = Self {
            dev_handler: None,
            payload: vec![0; reserved as usize],
        };
        Ok(pkbuf)
    }

    pub fn dev_handler_mut(&mut self) -> &mut Option<Arc<Mutex<dyn NetDev>>> {
        &mut self.dev_handler
    }

    // pub fn is_local(&self) -> Result<bool> {}
}

#[allow(unused)]
fn is_broadcast(mac: &[u8]) -> bool {
    (mac[0] & mac[1] & mac[2] & mac[3] & mac[4] & mac[5]) == 0xff
}

#[allow(unused)]
fn is_multicast(mac: &[u8]) -> bool {
    mac[0] & 0x01 != 0
}

#[allow(unused)]
fn is_local(mac: &[u8]) -> bool {
    mac[0] == 0x02 && mac[1] == 0x42
}

#[allow(unused)]
fn hwacmp(mac1: &[u8], mac2: &[u8]) -> bool {
    mac1[0] == mac2[0]
        && mac1[1] == mac2[1]
        && mac1[2] == mac2[2]
        && mac1[3] == mac2[3]
        && mac1[4] == mac2[4]
        && mac1[5] == mac2[5]
}
