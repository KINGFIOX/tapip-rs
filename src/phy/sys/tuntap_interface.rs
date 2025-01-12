use crate::{phy::Medium, wire::EthernetFrame};

use std::io;
use std::mem::MaybeUninit;
use std::os::fd::FromRawFd;
use std::os::unix::io::{AsRawFd, RawFd};

#[derive(Debug)]
pub struct TunTapInterfaceDesc {
    lower: libc::c_int,
    mtu: usize,
}

impl AsRawFd for TunTapInterfaceDesc {
    fn as_raw_fd(&self) -> RawFd {
        self.lower
    }
}

// # Panics
// if name is longer than libc::IF_NAMESIZE
fn ifreq_for(name: &str) -> libc::ifreq {
    if name.len() > libc::IF_NAMESIZE {
        panic!("name is longer than libc::IF_NAMESIZE");
    }
    let mut ifr = unsafe { MaybeUninit::<libc::ifreq>::zeroed().assume_init() };
    for (i, byte) in name.as_bytes().iter().enumerate() {
        ifr.ifr_name[i] = *byte as libc::c_char
    }
    ifr
}

fn ifreq_add_flags(ifr: &mut libc::ifreq, flags: &[libc::c_int]) {
    unsafe {
        ifr.ifr_ifru.ifru_flags = 0; // clear flags
        for flag in flags {
            let flag = *flag as libc::c_short;
            ifr.ifr_ifru.ifru_flags |= flag;
        }
    }
}

fn ifreq_get_flags(lower: libc::c_int, ifr: &mut libc::ifreq) -> io::Result<()> {
    ifr.ifr_ifru.ifru_flags = 0;
    ifreq_ioctl(lower, ifr, libc::SIOCGIFFLAGS).map(|_| ())
}

fn ifreq_ioctl(
    lower: libc::c_int,
    ifr: &mut libc::ifreq,
    cmd: libc::c_ulong,
) -> io::Result<libc::c_int> {
    let res = unsafe { libc::ioctl(lower, cmd as _, ifr as *mut libc::ifreq) };
    if res == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(res)
}

fn socket(
    domain: libc::c_int,
    ty: libc::c_int,
    protocol: libc::c_int,
) -> io::Result<std::fs::File> {
    unsafe {
        let lower = libc::socket(domain, ty, protocol);
        if lower == -1 {
            return Err(io::Error::last_os_error());
        }
        Ok(std::fs::File::from_raw_fd(lower))
    }
}

impl TunTapInterfaceDesc {
    pub fn new(name: &str, medium: Medium) -> io::Result<TunTapInterfaceDesc> {
        let lower = unsafe {
            let lower = libc::open(
                "/dev/net/tun\0".as_ptr() as *const libc::c_char,
                libc::O_RDWR | libc::O_NONBLOCK,
            );
            if lower == -1 {
                return Err(io::Error::last_os_error());
            }
            lower
        };

        let mut ifreq = ifreq_for(name);
        Self::attach_interface_ifreq(lower, medium, &mut ifreq)?;
        let mtu = Self::mtu_ifreq(medium, &mut ifreq)?;
        Self::setup_ifreq(&mut ifreq)?;

        Ok(TunTapInterfaceDesc { lower, mtu })
    }

    pub fn from_fd(fd: RawFd, mtu: usize) -> io::Result<TunTapInterfaceDesc> {
        Ok(TunTapInterfaceDesc { lower: fd, mtu })
    }

    fn attach_interface_ifreq(
        lower: libc::c_int,
        medium: Medium,
        ifr: &mut libc::ifreq,
    ) -> io::Result<()> {
        let mode = match medium {
            Medium::Ethernet => libc::IFF_TAP,
        };
        ifreq_add_flags(ifr, &[mode, libc::IFF_NO_PI]);
        ifreq_ioctl(lower, ifr, libc::TUNSETIFF).map(|_| ())
    }

    fn setup_ifreq(ifr: &mut libc::ifreq) -> io::Result<()> {
        let sk = socket(libc::AF_INET, libc::SOCK_DGRAM, libc::IPPROTO_IP)?;
        ifreq_get_flags(sk.as_raw_fd(), ifr)?;
        ifreq_add_flags(ifr, &[libc::IFF_UP, libc::IFF_RUNNING]);
        ifreq_ioctl(sk.as_raw_fd(), ifr, libc::SIOCSIFFLAGS).map(|_| ())
    }

    fn mtu_ifreq(medium: Medium, ifr: &mut libc::ifreq) -> io::Result<usize> {
        let lower = socket(libc::AF_INET, libc::SOCK_DGRAM, libc::IPPROTO_IP)?;
        // Propagate error after close, to ensure we always close.
        let ip_mtu =
            ifreq_ioctl(lower.as_raw_fd(), ifr, libc::SIOCGIFMTU).map(|mtu| mtu as usize)?;

        // SIOCGIFMTU returns the IP MTU (typically 1500 bytes.)
        // smoltcp counts the entire Ethernet packet in the MTU, so add the Ethernet header size to it.
        let mtu = match medium {
            Medium::Ethernet => ip_mtu + EthernetFrame::<&[u8]>::header_len(),
        };

        Ok(mtu)
    }

    pub fn interface_mtu(&self) -> io::Result<usize> {
        Ok(self.mtu)
    }

    pub fn recv(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        unsafe {
            let len = libc::read(
                self.lower,
                buffer.as_mut_ptr() as *mut libc::c_void,
                buffer.len(),
            );
            if len == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok(len as usize)
        }
    }

    pub fn send(&mut self, buffer: &[u8]) -> io::Result<usize> {
        unsafe {
            let len = libc::write(
                self.lower,
                buffer.as_ptr() as *const libc::c_void,
                buffer.len(),
            );
            if len == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok(len as usize)
        }
    }
}

impl Drop for TunTapInterfaceDesc {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.lower);
        }
    }
}
