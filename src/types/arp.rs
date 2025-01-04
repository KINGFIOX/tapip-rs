use super::*;

#[derive(Debug)]
#[repr(packed)]
pub struct Arp {
    hrd: be16,           /* hardware address type */
    pro: be16,           /* protocol address type */
    hrdlen: u8,          /* hardware address length */
    prolen: u8,          /* protocol address length */
    op: be16,            /* ARP opcode(command) */
    sha: [u8; ETH_ALEN], /* sender hw addr. source hardware address */
    sip: be32,           /* sender ip addr */
    tha: [u8; ETH_ALEN], /* target hw addr */
    tip: be32,           /* target ip addr */
}

impl Arp {
    pub fn hrd(&self) -> u16 {
        self.hrd.into()
    }
    pub fn pro(&self) -> u16 {
        self.pro.into()
    }
    pub fn op(&self) -> u16 {
        self.op.into()
    }
}

impl From<&[u8]> for Arp {
    fn from(value: &[u8]) -> Self {
        let ptr = value.as_ptr() as *const Self;
        unsafe { ptr.read_unaligned() }
    }
}
