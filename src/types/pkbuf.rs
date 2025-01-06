use crate::netdev::NetDev;

use anyhow::Result;
use lazy_static::lazy_static;
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

lazy_static! {
    pub static ref ALLOC_PKGS: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketBufferType {
    Other,
    BoardCast,
    Multicast,
    Local,
}

pub struct PacketBuffer {
    dev_handler: Option<Arc<Mutex<dyn NetDev>>>,
    pub data: Vec<u8>,
    /// destination type
    pk_type: Option<PacketBufferType>,
    eth_pro: Option<u16>,
}

impl Debug for PacketBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PacketBuffer")
            .field("payload", &self.data)
            .field("pk_type", &self.pk_type)
            .field("eth_pro", &self.eth_pro)
            .finish()
    }
}

impl PacketBuffer {
    pub fn new(reserved: u16) -> Result<Self> {
        let mut allo = ALLOC_PKGS.lock().unwrap();
        *allo += 1;
        let pkbuf = Self {
            dev_handler: None,
            data: vec![0; reserved as usize],
            pk_type: None,
            eth_pro: None,
        };
        Ok(pkbuf)
    }

    pub fn dev_handler(&self) -> Option<Arc<Mutex<dyn NetDev>>> {
        self.dev_handler.clone()
    }

    pub fn dev_handler_mut(&mut self) -> &mut Option<Arc<Mutex<dyn NetDev>>> {
        &mut self.dev_handler
    }

    pub fn pk_type(&self) -> Option<PacketBufferType> {
        self.pk_type
    }

    pub fn pk_type_mut(&mut self) -> &mut Option<PacketBufferType> {
        &mut self.pk_type
    }

    pub fn eth_pro(&self) -> Option<u16> {
        self.eth_pro
    }

    pub fn eth_pro_mut(&mut self) -> &mut Option<u16> {
        &mut self.eth_pro
    }
}

impl PacketBuffer {
    pub fn payload<T>(&self) -> &T {
        let payload = self.data.as_slice() as *const _ as *const u8 as usize;
        let ptr = payload as *const T;
        unsafe { &*ptr }
    }
}
