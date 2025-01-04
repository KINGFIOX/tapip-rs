use super::*;

#[derive(Debug)]
#[repr(packed)]
pub struct Ipv4 {
    /// ip_hlen[3:0], ip_ver[7:4]
    _version: u8,
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
    src: be32,
    dst: be32,
}

impl Ipv4 {
    pub fn hlen(&self) -> usize {
        (self._version & 0x0F) as usize
    }

    pub fn version(&self) -> u8 {
        (self._version & 0xF0) >> 4
    }

    pub fn len(&self) -> usize {
        let len: u16 = self.len.into();
        len as usize
    }

    pub fn payload(&self) -> &[u8] {
        let ptr = self as *const _ as usize;
        let ppayload = (ptr + self.hlen()) as *const u8;
        let len = self.len() - self.hlen();
        unsafe { std::slice::from_raw_parts(ppayload, len) }
    }
}
