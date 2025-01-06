use super::*;

pub const IP_HRD_SZ: usize = size_of::<IPV4Hdr>();

pub const IP_VERSION_4: u8 = 4;

#[repr(transparent)]
#[derive(Clone, Copy)]
struct VerHlen(u8);

impl Debug for VerHlen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "version: {}, hlen:{}", self.version(), self.hlen())
    }
}

impl VerHlen {
    pub fn hlen(&self) -> usize {
        (self.0 & 0x0F) as usize
    }

    pub fn version(&self) -> u8 {
        (self.0 & 0xF0) >> 4
    }
}

#[allow(unused)]
#[derive(Debug)]
#[repr(packed)]
pub struct IPV4Hdr {
    /// ip_hlen[3:0], ip_ver[7:4]
    ver_hlen: VerHlen,
    /// type of service
    tos: u8,
    len: be16,
    id: be16,
    /// fragment offset(in 8-octet's)
    frag_off: be16,
    ttl: u8,
    /// udp, tcp, icmp, etc.
    protocol: u8,
    checksum: be16,
    src: IPV4Addr,
    dst: IPV4Addr,
}

impl IPV4Hdr {
    pub fn hlen(&self) -> usize {
        self.ver_hlen.hlen()
    }

    pub fn version(&self) -> u8 {
        self.ver_hlen.version()
    }

    pub fn len(&self) -> usize {
        let len: u16 = self.len.into();
        len as usize
    }

    #[allow(unused)]
    pub fn payload(&self) -> &[u8] {
        let ptr = self as *const _ as usize;
        let ppayload = (ptr + self.hlen()) as *const u8;
        let len = self.len() - self.hlen();
        unsafe { std::slice::from_raw_parts(ppayload, len) }
    }

    #[allow(unused)]
    pub fn checksum(&self) -> u16 {
        self.checksum.into()
    }
}
