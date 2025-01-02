use super::*;
use lazy_static::lazy_static;
use libc::*;
use log::{error, info, warn};
use pkbuf::PacketBuffer;
use std::io::Error;
use std::sync::{Arc, Mutex};
use utils::ifrname::build_terminated_if_name;

lazy_static! {
    pub static ref VETH: Arc<Mutex<TapDev>> = Arc::new(Mutex::new(TapDev::new("tap0")));
}

const TUNTAPDEV: &str = "/dev/net/tun\0";

#[allow(unused)]
const TAP0: &str = "tap0\0";

const MTU: u16 = 1500;

#[derive(Debug)]
pub struct TapDev {
    fd: i32,
    mtu: u16,
    stats: NetStats,
}

impl TapDev {
    pub fn new(name: &str) -> Self {
        let fd = unsafe { open(TUNTAPDEV.as_ptr() as *const c_char, O_RDWR) };
        if fd < 0 {
            let err = Error::from_raw_os_error(-fd as i32);
            panic!("open failed: {}", err);
        }
        let ifr = ifreq {
            ifr_name: build_terminated_if_name(name),
            ifr_ifru: __c_anonymous_ifr_ifru {
                ifru_flags: (IFF_TAP | IFF_NO_PI) as _,
            },
        };
        let ret = unsafe { ioctl(fd, TUNSETIFF, &ifr) }; // FIXME: should be run as root
        if ret < 0 {
            unsafe { close(fd) };
            let err = Error::from_raw_os_error(-ret as i32);
            panic!("open failed: {}", err);
        }
        Self {
            fd,
            mtu: MTU,
            stats: NetStats::default(),
        }
    }
}

fn result(ret: isize) -> Result<usize> {
    if ret < 0 {
        todo!()
    }
    Ok(ret as usize)
}

impl NetDev for TapDev {
    fn xmit(&mut self, buf: &[u8]) -> Result<usize> {
        let ret = unsafe { write(self.fd, buf.as_ptr() as *const c_void, buf.len()) };
        result(ret)
    }
    fn recv(&mut self, buf: &mut [u8]) -> Result<usize> {
        let ret = unsafe { read(self.fd, buf.as_mut_ptr() as *mut c_void, buf.len()) };
        if ret <= 0 {
            self.stats.rx_errors += 1;
            let err = Error::from_raw_os_error(-ret as i32);
            warn!("read failed: {}", err);
            return Err(err).with_context(|| context!());
        }
        info!("read success: {}", ret);
        self.stats.rx_packets += 1;
        self.stats.rx_bytes += ret as u64;
        Ok(ret as usize)
    }
}

impl TapDev {
    fn alloc_pkbuf(&self) -> PacketBuffer {
        let mtu = self.mtu;
        PacketBuffer::new(mtu)
    }

    fn veth_rx(&mut self) {
        let mut pkbuf = self.alloc_pkbuf();
        if self.recv(&mut pkbuf.payload).is_ok() {
            todo!()
        }
    }

    pub fn veth_poll(&mut self) {
        loop {
            let mut pfd = pollfd {
                fd: self.fd,
                events: POLLIN,
                revents: 0,
            };
            let ret = unsafe { poll(&mut pfd, 1, -1) };
            if ret <= 0 {
                error!("poll failed: {}", ret);
            }
            self.veth_rx();
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused)]
    use super::*;

    #[test]
    fn test_open() {
        let dev = TapDev::new(TAP0);
        println!("{:?}", dev);
        assert!(dev.fd > 0);
    }

    #[test]
    fn io_error() {
        let err = std::io::Error::from_raw_os_error(5); // EIO
        println!("{:?}", err);
        let err = std::io::Error::from_raw_os_error(11); // EAGAIN
        println!("{:?}", err);
        let err = std::io::Error::from_raw_os_error(10); // no child
        println!("{:?}", err);
    }
}
