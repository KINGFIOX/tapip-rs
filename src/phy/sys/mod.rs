#![allow(unsafe_code)]

use crate::time::Duration;
use std::os::unix::io::RawFd;
use std::{io, mem, ptr};

pub mod tuntap_interface;

pub use self::tuntap_interface::TunTapInterfaceDesc;

/// Wait until given file descriptor becomes readable, but no longer than given timeout.
pub fn wait(fd: RawFd, duration: Option<Duration>) -> io::Result<()> {
    unsafe {
        let mut readfds = {
            let mut readfds = mem::MaybeUninit::<libc::fd_set>::uninit(); // readfds <- fd_set
            libc::FD_ZERO(readfds.as_mut_ptr()); // readfds <- {}
            libc::FD_SET(fd, readfds.as_mut_ptr()); // readfds U= {fd}
            readfds.assume_init() // declare that: readfds has been initialized
        };

        let mut writefds = {
            let mut writefds = mem::MaybeUninit::<libc::fd_set>::uninit();
            libc::FD_ZERO(writefds.as_mut_ptr());
            writefds.assume_init()
        };

        // exception fds
        let mut exceptfds = {
            let mut exceptfds = mem::MaybeUninit::<libc::fd_set>::uninit();
            libc::FD_ZERO(exceptfds.as_mut_ptr());
            exceptfds.assume_init()
        };

        let mut timeout = libc::timeval {
            tv_sec: 0,
            tv_usec: 0,
        };
        // set timeout
        let timeout_ptr = if let Some(duration) = duration {
            timeout.tv_sec = duration.secs() as libc::time_t;
            timeout.tv_usec = (duration.millis() * 1_000) as libc::suseconds_t;
            &mut timeout as *mut _
        } else {
            ptr::null_mut() // NULL ptr
        };

        let res = libc::select(
            fd + 1,
            &mut readfds,
            &mut writefds,
            &mut exceptfds,
            timeout_ptr,
        );
        if res == -1 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}
