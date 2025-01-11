/// A read/write wrapper around an Ethernet II frame buffer.
#[derive(Debug, Clone)]
pub struct Frame<T: AsRef<[u8]>> {
    #[allow(unused)]
    buffer: T,
}

#[allow(unused)]
mod field {
    use crate::wire::field::*;

    pub const DESTINATION: Field = 0..6;
    pub const SOURCE: Field = 6..12;
    pub const ETHERTYPE: Field = 12..14;
    pub const PAYLOAD: Rest = 14..;
}

pub const HEADER_LEN: usize = field::PAYLOAD.start;

impl<T: AsRef<[u8]>> Frame<T> {
    pub const fn header_len() -> usize {
        HEADER_LEN
    }
}

/// A six-octet Ethernet II address.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub struct Address(pub [u8; 6]);
