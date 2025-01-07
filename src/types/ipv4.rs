use utils::checksum;

use super::*;

pub const IP_HRD_SZ: usize = size_of::<Ipv4Header>();

pub const IP_VERSION_4: u8 = 4;

#[repr(transparent)]
#[derive(Clone, Copy)]
struct VerHlen(u8);

impl Debug for VerHlen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "version: {}, hlen:{}", self.version(), self.header_len())
    }
}

impl VerHlen {
    pub fn header_len(&self) -> usize {
        ((self.0 & 0x0F) as usize) << 2
    }

    pub fn version(&self) -> u8 {
        (self.0 & 0xF0) >> 4
    }
}

#[allow(unused)]
#[derive(Debug)]
#[repr(packed)]
pub struct Ipv4Header {
    /// ip_hlen[3:0], ip_ver[7:4]
    ver_hlen: VerHlen,
    /// type of service
    tos: u8,
    total_len: be16,
    /// identification
    ident: be16,
    /// fragment offset(in 8-octet's)
    frag_off: be16,
    /// time to live field.
    ttl: u8,
    /// udp, tcp, icmp, etc.
    protocol: u8,
    checksum: be16,
    src_addr: Ipv4Addr,
    dst_addr: Ipv4Addr,
}

/// getters
impl Ipv4Header {
    pub fn header_len(&self) -> usize {
        self.ver_hlen.header_len()
    }
    pub fn version(&self) -> u8 {
        self.ver_hlen.version()
    }
    pub fn total_len(&self) -> usize {
        let len: u16 = self.total_len.into();
        len as usize
    }
    #[allow(unused)]
    pub fn ident(&self) -> u16 {
        self.ident.into()
    }
    /// Return the time to live field.
    #[allow(unused)]
    pub fn hop_limit(&self) -> u8 {
        self.ttl
    }
    /// Return the next_header (protocol) field.
    #[allow(unused)]
    pub fn next_header(&self) -> u8 {
        self.protocol
    }
    #[allow(unused)]
    pub fn checksum(&self) -> u16 {
        self.checksum.into()
    }
    #[allow(unused)]
    pub fn src_addr(&self) -> Ipv4Addr {
        self.src_addr
    }
    #[allow(unused)]
    pub fn dst_addr(&self) -> Ipv4Addr {
        self.dst_addr
    }
    /// Validate the header checksum.
    ///
    /// # Fuzzing
    /// This function always returns `true` when fuzzing.
    pub fn verify_checksum(&self) -> bool {
        let this = self as *const Self as *const u8;
        let data = unsafe { std::slice::from_raw_parts(this, self.total_len() as usize) };
        checksum::data(&data[..self.header_len() as usize]) == !0 /* 0xffff */
    }
}

impl Ipv4Header {
    #[allow(unused)]
    pub fn payload(&self) -> &[u8] {
        let ptr = self as *const _ as usize;
        let ppayload = (ptr + self.header_len()) as *const u8;
        let len = self.total_len() - self.header_len();
        unsafe { std::slice::from_raw_parts(ppayload, len) }
    }
}
