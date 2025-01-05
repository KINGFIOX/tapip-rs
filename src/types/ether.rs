use hwa::HardwareAddr;

use super::*;

#[repr(packed)]
pub struct Ether {
    pub dst: HardwareAddr,
    pub src: HardwareAddr,
    pub protocol: be16,
}

impl From<&[u8]> for Ether {
    fn from(value: &[u8]) -> Self {
        let ptr = value.as_ptr() as *const Self;
        unsafe { ptr.read_unaligned() }
    }
}

impl Ether {
    pub fn payload(&self) -> *const u8 {
        let this = self as *const _ as usize;
        let ppayload = this + size_of::<Self>();
        ppayload as *const u8
    }
    pub fn protocol(&self) -> u16 {
        self.protocol.into()
    }
}
