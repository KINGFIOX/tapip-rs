use hwa::HardwareAddr;

use super::*;

pub const ARP_HRD_SZ: usize = size_of::<Arp>();
pub const ARP_HDR_ETHER: u16 = 1;
pub const ARP_OP_REQUEST: u16 = 1;
pub const ARP_OP_REPLY: u16 = 2;
pub const ARP_TIMEOUT: u32 = 600;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArpProtocol(be16);

impl Into<u16> for ArpProtocol {
    fn into(self) -> u16 {
        self.0.into()
    }
}

impl Debug for ArpProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let be = self.0;
        let le: u16 = be.into();
        write!(f, "0x{:04x}", le)
    }
}

#[derive(Debug)]
#[repr(packed)]
pub struct Arp {
    type_: be16,                        /* hardware address type */
    pro: ArpProtocol,                   /* protocol address type */
    hrd_len: u8,                        /* hardware address length */
    pro_len: u8,                        /* protocol address length */
    opcode: be16,                       /* ARP opcode(command) */
    src_hardware_addr: HardwareAddr,    /* sender hw addr. source hardware address */
    src_ip_addr: IPV4Addr,              /* sender ip addr */
    target_hardware_addr: HardwareAddr, /* target hw addr */
    target_ip_addr: IPV4Addr,           /* target ip addr */
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
    pub fn set_opcode(&mut self, opcode: u16) {
        self.opcode = be16::from_le(opcode);
    }
    pub fn src_hardware_addr(&self) -> HardwareAddr {
        let ptr = &self.src_hardware_addr as *const HardwareAddr;
        unsafe { ptr.read_unaligned() }
    }
    pub fn set_src_hardware_addr(&mut self, hardware_addr: HardwareAddr) {
        self.src_hardware_addr = hardware_addr;
    }
    pub fn set_src_ip_addr(&mut self, ip_addr: IPV4Addr) {
        self.src_ip_addr = ip_addr;
    }
    pub fn hdr_len(&self) -> u8 {
        self.hrd_len
    }
    pub fn pro_len(&self) -> u8 {
        self.pro_len
    }
    pub fn target_ip_addr(&self) -> IPV4Addr {
        self.target_ip_addr
    }
    pub fn set_target_ip_addr(&mut self, ip_addr: IPV4Addr) {
        self.target_ip_addr = ip_addr;
    }
    pub fn src_ip_addr(&self) -> IPV4Addr {
        self.src_ip_addr
    }
    pub fn set_target_hardware_addr(&mut self, hardware_addr: HardwareAddr) {
        self.target_hardware_addr = hardware_addr;
    }
}
