use super::*;

use rustix::io::{read, write};
use std::{fs::File, os::fd::AsFd};

pub struct TapDev {
    fd: File,
}

impl NetDev for TapDev {
    fn new(path: &str) -> Result<Rc<Self>> {
        let fd = File::open(path).with_context(|| context!())?;
        Ok(Rc::new(Self { fd }))
    }
    fn xmit(&self, buf: &[u8]) -> Result<usize> {
        write(self.fd.as_fd(), buf).with_context(|| context!())
    }
    fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        read(self.fd.as_fd(), buf).with_context(|| context!())
    }
}
