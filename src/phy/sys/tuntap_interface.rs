use crate::{phy::Medium, wire::EthernetFrame};
use std::io;
use std::os::fd::{AsRawFd, RawFd};

#[allow(unused_imports)]
use super::*;

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

impl TunTapInterfaceDesc {
    #[allow(unused)]
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

        Ok(TunTapInterfaceDesc { lower, mtu })
    }

    #[allow(unused)]
    pub fn from_fd(fd: RawFd, mtu: usize) -> io::Result<TunTapInterfaceDesc> {
        Ok(TunTapInterfaceDesc { lower: fd, mtu })
    }

    #[allow(unused)]
    pub fn interface_mtu(&self) -> io::Result<usize> {
        Ok(self.mtu)
    }

    fn attach_interface_ifreq(
        lower: libc::c_int,
        medium: Medium,
        ifr: &mut InterfaceRequest,
    ) -> io::Result<()> {
        let mode = match medium {
            Medium::Ip => imp::IFF_TUN,
            Medium::Ethernet => imp::IFF_TAP,
            Medium::Ieee802154 => todo!(),
        };
        ifr.ifr_data = mode | imp::IFF_NO_PI;
        ifreq_ioctl(lower, ifr, imp::TUNSETIFF).map(|_| ())
    }

    fn mtu_ifreq(medium: Medium, ifr: &mut InterfaceRequest) -> io::Result<usize> {
        let lower = unsafe {
            let lower = libc::socket(libc::AF_INET, libc::SOCK_DGRAM, libc::IPPROTO_IP);
            if lower == -1 {
                return Err(io::Error::last_os_error());
            }
            lower
        };

        let ip_mtu = ifreq_ioctl(lower, ifr, imp::SIOCGIFMTU).map(|mtu| mtu as usize);

        unsafe {
            libc::close(lower);
        }

        // Propagate error after close, to ensure we always close.
        let ip_mtu = ip_mtu?;

        // SIOCGIFMTU returns the IP MTU (typically 1500 bytes.)
        // smoltcp counts the entire Ethernet packet in the MTU, so add the Ethernet header size to it.
        let mtu = match medium {
            Medium::Ip => ip_mtu,
            Medium::Ethernet => ip_mtu + EthernetFrame::<&[u8]>::header_len(),
            Medium::Ieee802154 => todo!(),
        };

        Ok(mtu)
    }
}
