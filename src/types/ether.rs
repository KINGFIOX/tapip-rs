use super::*;

#[repr(packed)]
pub struct Ether {
    pub dst: [u8; ETH_ALEN],
    pub src: [u8; ETH_ALEN],
    pub ether_type: be16,
}

impl Ether {
    pub fn payload(&self) -> *const u8 {
        let this = self as *const _ as usize;
        let ppayload = this + size_of::<Self>();
        ppayload as *const u8
    }
    pub fn ether_type(&self) -> u16 {
        self.ether_type.into()
    }
}
