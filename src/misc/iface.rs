use super::*;

#[derive(Debug)]
pub struct Iface {
    fd: i32,
}

impl Iface {
    pub fn new(name: &str) -> Result<Self> {
        todo!()
    }

    pub fn send(&self, buf: &[u8]) -> Result<usize> {
        todo!()
    }

    pub fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        todo!()
    }
}
