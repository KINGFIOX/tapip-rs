pub mod tuntap_interface;
pub use self::tuntap_interface::TunTapInterfaceDesc;

use std::io;

#[path = "linux.rs"]
mod imp;

#[repr(C)]
#[derive(Debug)]
struct InterfaceRequest {
    ifr_name: [libc::c_char; libc::IF_NAMESIZE],
    ifr_data: libc::c_int, /* ifr_ifindex or ifr_mtu */
}

fn ifreq_ioctl(
    lower: libc::c_int,
    ifreq: &mut InterfaceRequest,
    cmd: libc::c_ulong,
) -> io::Result<libc::c_int> {
    unsafe {
        let res = libc::ioctl(lower, cmd as _, ifreq as *mut InterfaceRequest);
        if res == -1 {
            return Err(io::Error::last_os_error());
        }
    }

    Ok(ifreq.ifr_data)
}

fn ifreq_for(name: &str) -> InterfaceRequest {
    let mut ifreq = InterfaceRequest {
        ifr_name: [0; libc::IF_NAMESIZE],
        ifr_data: 0,
    };
    for (i, byte) in name.as_bytes().iter().enumerate() {
        ifreq.ifr_name[i] = *byte as libc::c_char
    }
    ifreq
}
