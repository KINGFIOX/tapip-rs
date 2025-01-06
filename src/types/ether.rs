use hwa::HardwareAddr;

use super::*;

#[repr(packed)]
#[derive(Debug)]
pub struct Ether {
    dst: HardwareAddr,
    src: HardwareAddr,
    protocol: be16,
}

impl Ether {
    pub fn dst(&self) -> HardwareAddr {
        self.dst
    }
    pub fn src(&self) -> HardwareAddr {
        self.src
    }
    pub fn protocol(&self) -> u16 {
        self.protocol.into()
    }
}

impl Ether {
    pub fn payload<T>(&self) -> &T {
        let this = self as *const Self as usize;
        let ppayload = this + size_of::<Self>();
        let ptr = ppayload as *const T;
        let obj = unsafe { &*ptr };
        obj
    }
}
