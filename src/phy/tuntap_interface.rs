use std::os::fd::RawFd;
use std::rc::Rc;
use std::{cell::RefCell, io};

use crate::phy::{self, sys, Medium};

/// A virtual TUN (IP) or TAP (Ethernet) interface.
#[derive(Debug)]
pub struct TunTapInterface {
    #[allow(unused)]
    lower: Rc<RefCell<sys::TunTapInterfaceDesc>>,
    #[allow(unused)]
    mtu: usize,
    #[allow(unused)]
    medium: Medium,
}

impl TunTapInterface {
    #[allow(unused)]
    /// Attaches to a TUN/TAP interface called `name`, or creates it if it does not exist.
    ///
    /// If `name` is a persistent interface configured with UID of the current user,
    /// no special privileges are needed. Otherwise, this requires superuser privileges
    /// or a corresponding capability set on the executable.
    pub fn new(name: &str, medium: Medium) -> io::Result<TunTapInterface> {
        let lower = sys::TunTapInterfaceDesc::new(name, medium)?;
        let mtu = lower.interface_mtu()?;
        Ok(TunTapInterface {
            lower: Rc::new(RefCell::new(lower)),
            mtu,
            medium,
        })
    }

    #[allow(unused)]
    /// Attaches to a TUN/TAP interface specified by file descriptor `fd`.
    ///
    /// On platforms like Android, a file descriptor to a tun interface is exposed.
    /// On these platforms, a TunTapInterface cannot be instantiated with a name.
    pub fn from_fd(fd: RawFd, medium: Medium, mtu: usize) -> io::Result<TunTapInterface> {
        let lower = sys::TunTapInterfaceDesc::from_fd(fd, mtu)?;
        Ok(TunTapInterface {
            lower: Rc::new(RefCell::new(lower)),
            mtu,
            medium,
        })
    }
}

pub struct RxToken {
    buffer: Vec<u8>,
}

impl phy::RxToken for RxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(&self.buffer[..])
    }
}

pub struct TxToken {
    lower: Rc<RefCell<sys::TunTapInterfaceDesc>>,
}

impl phy::TxToken for TxToken {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut lower = self.lower.borrow_mut();
        let mut buffer = vec![0; len];
        let result = f(&mut buffer);
        match lower.send(&buffer[..]) {
            Ok(_) => {}
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                // net_debug!("phy: tx failed due to WouldBlock")
            }
            Err(err) => panic!("{}", err),
        }
        result
    }
}
