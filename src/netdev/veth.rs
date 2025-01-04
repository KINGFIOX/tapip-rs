use super::*;
use lazy_static::lazy_static;
use log::{info, warn};
use net::net::net_in;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};
use tun_tap::Iface;
use types::pkbuf::PacketBuffer;

lazy_static! {
    pub static ref VETH: Arc<Mutex<TapDev>> = Arc::new(Mutex::new(TapDev::new("tun0").unwrap()));
}

#[derive(Debug)]
pub struct TapDev {
    iface: Iface,
    stats: NetStats,
}

impl TapDev {
    pub fn new(name: &str) -> Result<Self> {
        let nic = Iface::new(name, tun_tap::Mode::Tap).with_context(|| context!())?;
        let dev = Self {
            iface: nic,
            stats: NetStats::default(),
        };
        Ok(dev)
    }
}

impl NetDev for TapDev {
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
}

impl TapDev {
    fn alloc_pkbuf(this: Arc<Mutex<Self>>) -> Result<PacketBuffer> {
        let mut pkbuf = PacketBuffer::new(MTU + ETH_HRD_SZ + PACKET_INFO)?;
        *pkbuf.dev_handler_mut() = Some(this.clone());
        Ok(pkbuf)
    }

    fn veth_rx(this: Arc<Mutex<Self>>) -> Result<()> {
        let mut pkbuf = Self::alloc_pkbuf(this.clone()).with_context(|| context!())?;
        this.lock()
            .unwrap()
            .recv(&mut pkbuf.payload)
            .with_context(|| context!())?;
        let pkbuf = Rc::new(RefCell::new(pkbuf));
        net_in(pkbuf).with_context(|| context!())?;

        Ok(())
    }
}

impl TapDev {
    pub fn veth_poll(this: Arc<Mutex<Self>>) {
        loop {
            let ret = Self::veth_rx(this.clone());
            if let Err(e) = ret {
                warn!("veth poll error: {}", e);
            }
        }
    }
}

pub fn veth_poll() {
    TapDev::veth_poll(VETH.clone());
}

#[cfg(test)]
mod tests {
    #[allow(unused)]
    use super::*;

    #[test]
    fn it_works() {
        TapDev::veth_poll(VETH.clone());
    }
}
