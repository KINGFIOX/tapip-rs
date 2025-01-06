use hwa::HardwareAddr;

use super::*;

pub const ARP_HRD_SZ: usize = size_of::<Arp>();
pub const ARP_HDR_ETHER: u16 = 1;
pub const ARP_OP_REQUEST: u16 = 1;
pub const ARP_OP_REPLY: u16 = 2;

#[derive(Debug)]
#[repr(packed)]
pub struct Arp {
    type_: be16,                        /* hardware address type */
    pro: be16,                          /* protocol address type */
    hrd_len: u8,                        /* hardware address length */
    pro_len: u8,                        /* protocol address length */
    opcode: be16,                       /* ARP opcode(command) */
    src_hardware_addr: HardwareAddr,    /* sender hw addr. source hardware address */
    src_ip_addr: be32,                  /* sender ip addr */
    target_hardware_addr: HardwareAddr, /* target hw addr */
    target_ip_addr: be32,               /* target ip addr */
}

impl Arp {
    pub fn type_(&self) -> u16 {
        self.type_.into()
    }
    pub fn protocol(&self) -> u16 {
        self.pro.into()
    }
    pub fn opcode(&self) -> u16 {
        self.opcode.into()
    }
    pub fn src_hardware_addr(&self) -> HardwareAddr {
        self.src_hardware_addr
    }
    pub fn hdr_len(&self) -> u8 {
        self.hrd_len
    }
    pub fn pro_len(&self) -> u8 {
        self.pro_len
    }
}

impl From<&[u8]> for Arp {
    fn from(value: &[u8]) -> Self {
        let ptr = value.as_ptr() as *const Self;
        unsafe { ptr.read_unaligned() }
    }
}
