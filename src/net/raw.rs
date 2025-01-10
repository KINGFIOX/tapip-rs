use std::sync::{Arc, Mutex};

use super::*;
use types::{ipv4::Ipv4Protocol, pkbuf::PacketBuffer, Ipv4Addr};

// lazy_static! {
//     static ref RAW_SOCK: Arc<Mutex<>>
// }

struct Key {
    protocol: Ipv4Protocol,
    src_addr: Ipv4Addr,
    dst_addr: Ipv4Addr,
}

pub fn raw_in(_pkbuf: &mut PacketBuffer) -> Result<()> {
    todo!()
}
