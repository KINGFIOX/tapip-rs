#[allow(unused)]
use super::*;

use lazy_static::lazy_static;
use netdev::NetDev;
use std::sync::{Arc, Mutex};

lazy_static! {
    pub static ref ALLOC_PKGS: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
}

/// 一定得有一个 cache 用来存放 packet 的
/// - receive packet 是异步的, 一个后台线程轮询
/// - tcp 会有流量控制, 因此也一定要有一个 buffer
pub struct PacketBuffer {
    pub dev_handler: Arc<Mutex<dyn NetDev>>,
    pub payload: Vec<u8>,
}

impl PacketBuffer {
    pub fn new(reserved: u16, dev_handler: Arc<Mutex<dyn NetDev>>) -> Result<Self> {
        let mut allo = ALLOC_PKGS.lock().unwrap();
        *allo += 1;
        let pkbuf = Self {
            dev_handler,
            payload: vec![0; reserved as usize],
        };
        Ok(pkbuf)
    }
}
