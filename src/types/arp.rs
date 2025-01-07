//! # ARP protocol
//!
//! generally, the api is `xxx` as getter and `xxx_mut` as setter.
//! becase of the unaligned access, we could not use `xxx_mut` as setter,
//! so we use `set_xxx` as setter, instead.

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
    hardware_type: be16,                /* hardware address type */
    protocol_type: ArpProtocol,         /* protocol address type */
    hardware_len: u8,                   /* hardware address length */
    protocol_len: u8,                   /* protocol address length */
    operation: be16,                    /* ARP opcode(command) */
    source_hardware_addr: HardwareAddr, /* sender hw addr. source hardware address */
    source_ipv4_addr: IPV4Addr,         /* sender ip addr */
    target_hardware_addr: HardwareAddr, /* target hw addr */
    target_ipv4_addr: IPV4Addr,         /* target ip addr */
}

/// getters
impl Arp {
    pub fn hardware_type(&self) -> u16 {
        self.hardware_type.into()
    }
    pub fn protocol_type(&self) -> ArpProtocol {
        self.protocol_type
    }
    pub fn hardware_len(&self) -> u8 {
        self.hardware_len
    }
    pub fn protocol_len(&self) -> u8 {
        self.protocol_len
    }
    pub fn operation(&self) -> u16 {
        self.operation.into()
    }
    pub fn source_hardware_addr(&self) -> HardwareAddr {
        self.source_hardware_addr
    }
    pub fn source_ipv4_addr(&self) -> IPV4Addr {
        self.source_ipv4_addr
    }
    #[allow(unused)]
    pub fn target_hardware_addr(&self) -> HardwareAddr {
        self.target_hardware_addr
    }
    pub fn target_ipv4_addr(&self) -> IPV4Addr {
        self.target_ipv4_addr
    }
}

/// setters
impl Arp {
    pub fn set_operation(&mut self, operation: u16) {
        self.operation = be16::from_le(operation);
    }
    pub fn set_source_hardware_addr(&mut self, hardware_addr: HardwareAddr) {
        self.source_hardware_addr = hardware_addr;
    }
    pub fn set_source_ipv4_addr(&mut self, ip_addr: IPV4Addr) {
        self.source_ipv4_addr = ip_addr;
    }
    pub fn set_target_hardware_addr(&mut self, hardware_addr: HardwareAddr) {
        self.target_hardware_addr = hardware_addr;
    }
    pub fn set_target_ipv4_addr(&mut self, ip_addr: IPV4Addr) {
        self.target_ipv4_addr = ip_addr;
    }
}
