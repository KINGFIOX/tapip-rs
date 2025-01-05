use super::*;
use std::{cell::RefCell, rc::Rc};
use types::pkbuf::PacketBuffer;

fn eth_init(pkbuf: Rc<RefCell<PacketBuffer>>) {}

pub fn net_in(pkbuf: Rc<RefCell<PacketBuffer>>) -> Result<()> {
    eth_init(pkbuf.clone());
    loop {}
}
