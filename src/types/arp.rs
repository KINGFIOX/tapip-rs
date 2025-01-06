use hwa::HardwareAddr;

use super::*;

pub const ARP_HRD_SZ: usize = size_of::<Arp>();
pub const ARP_HDR_ETHER: u16 = 1;
pub const ARP_OP_REQUEST: u16 = 1;
pub const ARP_OP_REPLY: u16 = 2;
pub const ARP_TIMEOUT: u32 = 600;

pub type ArpProtocol = be16;

#[derive(Debug)]
#[repr(packed)]
pub struct Arp {
    type_: be16,                        /* hardware address type */
    pro: ArpProtocol,                   /* protocol address type */
    hrd_len: u8,                        /* hardware address length */
    pro_len: u8,                        /* protocol address length */
    opcode: be16,                       /* ARP opcode(command) */
    src_hardware_addr: HardwareAddr,    /* sender hw addr. source hardware address */
    src_ip_addr: IPAddr,                /* sender ip addr */
    target_hardware_addr: HardwareAddr, /* target hw addr */
    target_ip_addr: IPAddr,             /* target ip addr */
}

impl Arp {
    pub fn type_(&self) -> u16 {
        self.type_.into()
    }
    pub fn protocol(&self) -> ArpProtocol {
        self.pro
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
    pub fn target_ip_addr(&self) -> IPAddr {
        self.target_ip_addr
    }
    pub fn src_ip_addr(&self) -> IPAddr {
        self.src_ip_addr
    }
}

impl From<*const u8> for Arp {
    fn from(value: *const u8) -> Self {
        let ptr = value as *const Arp;
        unsafe { ptr.read_unaligned() }
    }
}
