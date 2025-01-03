use super::*;
use lazy_static::lazy_static;
use log::{info, warn};
use net::net_in;
use pkbuf::PacketBuffer;
use std::sync::{Arc, Mutex};
use tun_tap::Iface;

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
                self.stats.tx_packets += 1;
                self.stats.tx_bytes += n as u64;
                info!("tx success: {}", n);
                Ok(n)
            }
            Err(e) => {
                self.stats.tx_errors += 1;
                warn!("{}", e);
                Err(e)
            }
        }
    }
    fn recv(&mut self, buf: &mut [u8]) -> Result<usize> {
        let ret = self.iface.recv(buf).with_context(|| context!());
        match ret {
            Ok(n) => {
                self.stats.rx_packets += 1;
                self.stats.rx_bytes += n as u64;
                info!("rx success: {}", n);
                Ok(n)
            }
            Err(e) => {
                self.stats.rx_errors += 1;
                warn!("{}", e);
                Err(e)
            }
        }
    }
}

impl TapDev {
    fn alloc_pkbuf(this: Arc<Mutex<Self>>) -> Result<PacketBuffer> {
        PacketBuffer::new(MTU + ETH_HRD_SZ + PACKET_INFO, this.clone())
    }

    fn veth_rx(this: Arc<Mutex<Self>>) -> Result<()> {
        let mut pkbuf = Self::alloc_pkbuf(this.clone()).with_context(|| context!())?;
        this.lock()
            .unwrap()
            .recv(&mut pkbuf.payload)
            .with_context(|| context!())?;
        net_in(&pkbuf.payload).with_context(|| context!())?;

        Ok(())
    }
}

impl TapDev {
    pub fn veth_poll(this: Arc<Mutex<Self>>) {
        loop {
            Self::veth_rx(this.clone());
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
