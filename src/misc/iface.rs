use super::*;

use std::ffi::{c_char, c_int, c_uint, CString};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, FromRawFd};

use anyhow::Context;
use libc::{c_uchar, ETH_ALEN, IFNAMSIZ};

#[derive(Debug)]
pub struct Iface {
    interface_fd: File,
    mtu: i32,
    /// hardware address
    hardware_addr: [u8; ETH_ALEN as usize],
    ipv4_addr: u32,
}

extern "C" {
    fn set_tap_if(fd: c_int, name: *const c_char) -> c_int;
    fn set_tap(skfd: *mut c_int) -> c_int;
    fn getname_tap(tapfd: c_int, name: *mut c_char) -> c_int;
    fn getmtu_tap(skfd: c_int, name: *const c_char, mtu: *mut c_int) -> c_int;
    fn gethwaddr_tap(tapfd: c_int, ha: *mut c_uchar) -> c_int;
    fn setipaddr_tap(skfd: c_int, name: *const c_char, ipaddr: c_uint) -> c_int;
    fn getipaddr_tap(skfd: c_int, name: *const c_char, ipaddr: *mut c_uint) -> c_int;
    fn setnetmask_tap(skfd: c_int, name: *const c_char, netmask: c_uint) -> c_int;
    fn setup_tap(skfd: c_int, name: *const c_char) -> c_int;
}

impl Iface {
    pub fn new(name: &str) -> Result<Self> {
        let if_fd = OpenOptions::new() //
            .read(true)
            .write(true)
            .open("/dev/net/tun")?;
        let ptr: *const c_char = CString::new(name)?.as_ptr();
        // set interface to tap mode and set interface name
        let err = unsafe { set_tap_if(if_fd.as_raw_fd(), ptr) };
        if err != 0 {
            let err_str = std::io::Error::from_raw_os_error(err);
            return Err(anyhow::anyhow!("set_tap_if failed: {}", err_str))
                .with_context(|| context!());
        }
        // set socket fd
        let mut skfd = 0;
        let err = unsafe { set_tap(&mut skfd) };
        if err != 0 {
            let err_str = std::io::Error::from_raw_os_error(err);
            return Err(anyhow::anyhow!("set_tap failed: {}", err_str)).with_context(|| context!());
        }
        // raii
        let sk_fd = unsafe { File::from_raw_fd(skfd) };
        // get interface name, use it to getmtu_tap
        let mut if_name = [0; IFNAMSIZ];
        let err = unsafe { getname_tap(sk_fd.as_raw_fd(), if_name.as_mut_ptr()) };
        if err != 0 {
            let err_str = std::io::Error::from_raw_os_error(err);
            return Err(anyhow::anyhow!("getname_tap failed: {}", err_str))
                .with_context(|| context!());
        }
        // get mtu
        let mut mtu = 0;
        let err = unsafe { getmtu_tap(sk_fd.as_raw_fd(), if_name.as_ptr(), &mut mtu) };
        if err != 0 {
            let err_str = std::io::Error::from_raw_os_error(err);
            return Err(anyhow::anyhow!("getmtu_tap failed: {}", err_str))
                .with_context(|| context!());
        }
        // get hardware address
        let mut ha = [0; ETH_ALEN as usize];
        let err = unsafe { gethwaddr_tap(sk_fd.as_raw_fd(), ha.as_mut_ptr()) };
        if err != 0 {
            let err_str = std::io::Error::from_raw_os_error(err);
            return Err(anyhow::anyhow!("gethwaddr_tap failed: {}", err_str))
                .with_context(|| context!());
        }
        // set ipv4 address
        let err = unsafe {
            setipaddr_tap(
                sk_fd.as_raw_fd(),
                if_name.as_ptr(),
                0x02_00_00_0a, /*10.0.0.2*/
            )
        };
        if err != 0 {
            let err_str = std::io::Error::from_raw_os_error(err);
            return Err(anyhow::anyhow!("setipaddr_tap failed: {}", err_str))
                .with_context(|| context!());
        }
        // get ipv4 address
        let mut ipaddr = 0;
        let err = unsafe { getipaddr_tap(sk_fd.as_raw_fd(), if_name.as_ptr(), &mut ipaddr) };
        if err != 0 {
            let err_str = std::io::Error::from_raw_os_error(err);
            return Err(anyhow::anyhow!("getipaddr_tap failed: {}", err_str))
                .with_context(|| context!());
        }
        // set netmask
        let err = unsafe {
            setnetmask_tap(
                sk_fd.as_raw_fd(),
                if_name.as_ptr(),
                0x00_ff_ff_ff, /*255.255.255.0*/
            )
        };
        if err != 0 {
            let err_str = std::io::Error::from_raw_os_error(err);
            return Err(anyhow::anyhow!("setnetmask_tap failed: {}", err_str))
                .with_context(|| context!());
        }
        // setup interface
        let err = unsafe { setup_tap(sk_fd.as_raw_fd(), if_name.as_ptr()) };
        if err != 0 {
            let err_str = std::io::Error::from_raw_os_error(err);
            return Err(anyhow::anyhow!("setup_tap failed: {}", err_str))
                .with_context(|| context!());
        }
        Ok(Self {
            interface_fd: if_fd,
            mtu,
            hardware_addr: ha,
            ipv4_addr: ipaddr,
        })
    }

    pub fn send(&mut self, buf: &[u8]) -> Result<usize> {
        let l = self.interface_fd.write(buf).with_context(|| context!())?;
        Ok(l)
    }

    pub fn recv(&mut self, buf: &mut [u8]) -> Result<usize> {
        let l = self.interface_fd.read(buf).with_context(|| context!())?;
        Ok(l)
    }
}

#[cfg(test)]
mod tests {
    use libc::write;
    use std::{
        ffi::{c_void, CString, OsStr},
        os::unix::ffi::OsStrExt,
    };

    #[test]
    fn it_works() {
        let name1 = "tun1\n";
        let name2 = OsStr::new(name1);
        let name3 = CString::new(name1).expect("CString::new failed");
        println!("{:?}", name2.as_bytes());
        println!("{:?}", name3.as_bytes());
        unsafe { write(0, name3.as_ptr() as *const c_void, 6) };
        println!("{}", name3.count_bytes());
    }
}
