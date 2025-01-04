pub mod arp;
pub mod ether;
pub mod ip;
pub mod pkbuf;

pub const ETH_ALEN: usize = 6;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub struct be16(u16);

impl Into<u16> for be16 {
    fn into(self) -> u16 {
        u16::from_be(self.0)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub struct be32(u32);
