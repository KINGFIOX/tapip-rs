use super::*;

use std::ffi::{c_char, c_int, CString};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, FromRawFd};

use anyhow::Context;
use libc::{c_uchar, ETH_ALEN, IFNAMSIZ};
use types::hwa::HardwareAddr;
use types::{FromLe, Ipv4Addr};

#[derive(Debug)]
pub struct Iface {
    interface_fd: File,
    #[allow(unused)]
    mtu: i32,
    /// hardware address
    hardware_addr: HardwareAddr,
    ipv4_addr: Ipv4Addr,
}

extern "C" {
    fn set_tap_if(fd: c_int, name: *const c_char) -> c_int;
    fn set_tap(skfd: *mut c_int) -> c_int;
    fn getname_tap(tapfd: c_int, name: *mut c_char) -> c_int;
    fn getmtu_tap(skfd: c_int, name: *const c_char, mtu: *mut c_int) -> c_int;
    fn gethwaddr_tap(tapfd: c_int, ha: *mut c_uchar) -> c_int;
    fn setipaddr_tap(skfd: c_int, name: *const c_char, ipaddr: Ipv4Addr) -> c_int;
    fn getipaddr_tap(skfd: c_int, name: *const c_char, ipaddr: *mut Ipv4Addr) -> c_int;
    fn setnetmask_tap(skfd: c_int, name: *const c_char, netmask: Ipv4Addr) -> c_int;
    fn setup_tap(skfd: c_int, name: *const c_char) -> c_int;
}

const IPV4_ADDR: u32 = 0x0a_00_00_02; /*10.0.0.2*/
const NETMASK: u32 = 0xff_ff_ff_00; /*255.255.255.0*/
const TUN_PATH: &str = "/dev/net/tun";

macro_rules! call_c_func {
    ($func:expr) => {
        let err = unsafe { $func };
        if err != 0 {
            let err_str = std::io::Error::from_raw_os_error(err);
            return Err(anyhow::anyhow!("{} failed: {}", stringify!($func), err_str))
                .with_context(|| context!());
        }
    };
}

impl Iface {
    /// same effect as the following commands in bash.
    /// ```bash
    /// ip addr add 10.0.0.2/24 dev tun0
    /// ip tuntap add tun0 mode tap
    /// ip link set up dev tun0
    /// ```
    pub fn new(name: &str) -> Result<Self> {
        let if_fd = OpenOptions::new() //
            .read(true)
            .write(true)
            .open(TUN_PATH)?;
        let name_cstr = CString::new(name)?;
        let ptr = name_cstr.as_ptr();

        // set interface to tap mode and set interface name
        call_c_func!(set_tap_if(if_fd.as_raw_fd(), ptr));
        // set socket fd
        let mut skfd = 0;
        call_c_func!(set_tap(&mut skfd));
        // sk_fd (RAII), which is used to get some metadata of the interface
        let sk_fd = unsafe { File::from_raw_fd(skfd) };
        // get interface name, use it to getmtu_tap
        let mut if_name = [0; IFNAMSIZ];
        call_c_func!(getname_tap(if_fd.as_raw_fd(), if_name.as_mut_ptr()));
        // get mtu
        let mut mtu = 0;
        call_c_func!(getmtu_tap(sk_fd.as_raw_fd(), if_name.as_ptr(), &mut mtu));
        // get hardware address
        let mut ha = [0; ETH_ALEN as usize];
        call_c_func!(gethwaddr_tap(if_fd.as_raw_fd(), ha.as_mut_ptr()));
        // set ipv4 address
        call_c_func!(setipaddr_tap(
            sk_fd.as_raw_fd(),
            if_name.as_ptr(),
            Ipv4Addr::from_le(IPV4_ADDR), /*10.0.0.2*/
        ));
        // get ipv4 address
        let mut ipaddr: Ipv4Addr = Ipv4Addr::from_le(0); // big endian
        call_c_func!(getipaddr_tap(
            sk_fd.as_raw_fd(),
            if_name.as_ptr(),
            &mut ipaddr
        ));

        // set netmask
        call_c_func!(setnetmask_tap(
            sk_fd.as_raw_fd(),
            if_name.as_ptr(),
            Ipv4Addr::from_le(NETMASK),
        ));
        // setup interface
        call_c_func!(setup_tap(sk_fd.as_raw_fd(), if_name.as_ptr()));

        Ok(Self {
            interface_fd: if_fd,
            mtu,
            hardware_addr: HardwareAddr::from(ha),
            ipv4_addr: ipaddr,
        })
    }

    pub fn send(&mut self, buf: &[u8]) -> Result<usize> {
        self.interface_fd.write(buf).with_context(|| context!())
    }

    pub fn recv(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.interface_fd.read(buf).with_context(|| context!())
    }
}

impl Iface {
    pub fn hardware_addr(&self) -> HardwareAddr {
        self.hardware_addr
    }

    pub fn ipv4_addr(&self) -> Ipv4Addr {
        self.ipv4_addr
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
