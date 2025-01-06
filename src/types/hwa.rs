use std::fmt::Debug;

use libc::ETH_ALEN;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct HardwareAddr([u8; ETH_ALEN as usize]);

impl From<[u8; ETH_ALEN as usize]> for HardwareAddr {
    fn from(value: [u8; ETH_ALEN as usize]) -> Self {
        Self(value)
    }
}

impl HardwareAddr {
    pub fn is_broadcast(&self) -> bool {
        (self.0[0] & self.0[1] & self.0[2] & self.0[3] & self.0[4] & self.0[5]) == 0xff
    }
    pub fn is_multicast(&self) -> bool {
        self.0[0] & 0x01 != 0
    }
    pub fn is_local(&self) -> bool {
        self.0[0] == 0x02 && self.0[1] == 0x42
    }
}

impl Debug for HardwareAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}
