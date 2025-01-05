pub mod arp;
pub mod ether;
pub mod hwa;
pub mod ip;
pub mod pkbuf;

#[repr(transparent)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub struct be16(u16);

impl Into<u16> for be16 {
    fn into(self) -> u16 {
        u16::from_be(self.0)
    }
}

#[repr(transparent)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub struct be32(u32);
