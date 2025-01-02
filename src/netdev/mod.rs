use std::rc::Rc;

use super::*;
use anyhow::{Context, Result};

mod veth;

#[allow(unused)]
pub trait NetDev {
    fn xmit(&self, buf: &[u8]) -> Result<usize>;
    fn recv(&self, buf: &mut [u8]) -> Result<usize>;
    fn new(path: &str) -> Result<Rc<Self>>;
}
