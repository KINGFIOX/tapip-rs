use super::*;
use lazy_static::lazy_static;
use log::{info, warn};
use misc::iface::Iface;
use net::net::net_in;
use std::sync::{Arc, Mutex};
use types::{pkbuf::PacketBuffer, IPV4Addr};

lazy_static! {
    pub static ref VETH: Arc<Mutex<VethDev>> = Arc::new(Mutex::new(VethDev::new("tun0").unwrap()));
}

#[derive(Debug)]
pub struct VethDev {
    iface: Iface,
    stats: NetStats,
}

impl VethDev {
    pub fn new(name: &str) -> Result<Self> {
        let nic = Iface::new(name).with_context(|| context!())?;
        let dev = Self {
            iface: nic,
            stats: NetStats::default(),
        };
        Ok(dev)
    }
}

impl NetDev for VethDev {
    fn xmit(&mut self, buf: &[u8]) -> Result<usize> {
        let ret = self.iface.send(buf).with_context(|| context!());
        match ret {
            Ok(n) => {
                self.stats.tx.packets += 1;
                self.stats.tx.bytes += n as u64;
                info!("tx success: {}", n);
                Ok(n)
            }
            Err(e) => {
                self.stats.tx.errors += 1;
                warn!("{}", e);
                Err(e)
            }
        }
    }
    fn recv(&mut self, buf: &mut [u8]) -> Result<usize> {
        let ret = self.iface.recv(buf).with_context(|| context!());
        match ret {
            Ok(n) => {
                self.stats.rx.packets += 1;
                self.stats.rx.bytes += n as u64;
                info!("rx success: {}", n);
                Ok(n)
            }
            Err(e) => {
                self.stats.rx.errors += 1;
                warn!("{}", e);
                Err(e)
            }
        }
    }
    fn hardware_addr(&self) -> HardwareAddr {
        self.iface.hardware_addr()
    }
    fn ipv4_addr(&self) -> IPV4Addr {
        self.iface.ipv4_addr()
    }
}

impl VethDev {
    fn alloc_pkbuf(this: Arc<Mutex<Self>>) -> Result<PacketBuffer> {
        let mut pkbuf = PacketBuffer::new(MTU + ETH_HRD_SZ)?;
        *pkbuf.dev_handler_mut() = Some(this.clone());
        Ok(pkbuf)
    }

    fn veth_rx(this: Arc<Mutex<Self>>) -> Result<()> {
        let mut pkbuf = Self::alloc_pkbuf(this.clone()).with_context(|| context!())?;
        this.lock()
            .unwrap()
            .recv(pkbuf.data_mut())
            .with_context(|| context!())?;
        let pkbuf = Box::new(pkbuf);
        net_in(pkbuf).with_context(|| context!())?;

        Ok(())
    }
}

impl VethDev {
    pub fn veth_poll(this: Arc<Mutex<Self>>) {
        loop {
            let ret = Self::veth_rx(this.clone());
            if ret.is_err() {
                warn!("veth poll error: {:?}", ret);
            }
        }
    }
}

pub fn veth_poll() {
    VethDev::veth_poll(VETH.clone());
}
