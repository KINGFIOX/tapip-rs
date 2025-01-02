#[allow(unused)]
use super::*;

use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

lazy_static! {
    pub static ref ALLOC_PKGS: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
}

pub struct PacketBuffer {
    pub payload: Vec<u8>,
}

impl PacketBuffer {
    pub fn new(reserved: u16) -> Self {
        let mut allo = ALLOC_PKGS.lock().unwrap();
        *allo += 1;
        Self {
            payload: vec![0; reserved as usize],
        }
    }
}
