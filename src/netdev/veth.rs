use super::*;
use anyhow::Context;
use libc::*;
use log::{info, warn};
use std::io::Error;

pub struct TapDev {
    fd: i32,
    stats: NetStats,
}

fn result(ret: isize) -> Result<usize> {
    if ret < 0 {}
    Ok(ret as usize)
}

impl NetDev for TapDev {
    fn xmit(&mut self, buf: &[u8]) -> Result<usize> {
        let ret = unsafe { write(self.fd, buf.as_ptr() as *const c_void, buf.len()) };
        result(ret)
    }
    fn recv(&mut self, buf: &mut [u8]) -> Result<usize> {
        let ret = unsafe { read(self.fd, buf.as_mut_ptr() as *mut c_void, buf.len()) };
        let err = Error::from_raw_os_error(-ret as i32);
        if ret <= 0 {
            warn!("read failed: {}", err);
            self.stats.rx_errors += 1;
            return Err(err).with_context(|| context!());
        }
        info!("read success: {}", ret);
        self.stats.rx_packets += 1;
        self.stats.rx_bytes += ret as u64;
        Ok(ret as usize)
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused)]
    use super::*;

    #[test]
    fn it_works() {
        let err = std::io::Error::from_raw_os_error(5); // EIO
        println!("{:?}", err);
        let err = std::io::Error::from_raw_os_error(11); // EAGAIN
        println!("{:?}", err);
        let err = std::io::Error::from_raw_os_error(10); // no child
        println!("{:?}", err);
    }
}
