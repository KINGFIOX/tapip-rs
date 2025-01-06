use hwa::HardwareAddr;

use super::*;

#[repr(packed)]
#[derive(Debug)]
pub struct EtherHdr {
    dst: HardwareAddr,
    src: HardwareAddr,
    protocol: be16,
}

impl EtherHdr {
    pub fn dst(&self) -> HardwareAddr {
        self.dst
    }
    pub fn set_dst(&mut self, dst: HardwareAddr) {
        self.dst = dst;
    }
    pub fn src(&self) -> HardwareAddr {
        self.src
    }
    pub fn protocol(&self) -> u16 {
        self.protocol.into()
    }
    pub fn set_protocol(&mut self, protocol: u16) {
        self.protocol = be16::from_le(protocol);
    }
}

impl EtherHdr {
    pub fn payload<T>(&self) -> &T {
        let this = self as *const Self as usize;
        let ppayload = this + size_of::<Self>();
        let ptr = ppayload as *const T;
        unsafe { &*ptr }
    }

    pub fn payload_mut<T>(&mut self) -> &mut T {
        let this = self as *mut Self as usize;
        let ppayload = this + size_of::<Self>();
        let ptr = ppayload as *mut T;
        unsafe { &mut *ptr }
    }
}
