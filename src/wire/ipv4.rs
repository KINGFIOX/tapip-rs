use core::fmt;
pub use core::net::Ipv4Addr as Address;

use byteorder::{ByteOrder, NetworkEndian};

use super::IpProtocol as Protocol;
use super::{Error, Result};

pub const ADDR_SIZE: usize = 4;

mod field {
    use crate::wire::field::*;

    pub const VER_IHL: usize = 0;
    pub const DSCP_ECN: usize = 1;
    pub const LENGTH: Field = 2..4;
    pub const IDENT: Field = 4..6;
    pub const FLG_OFF: Field = 6..8;
    pub const TTL: usize = 8;
    pub const PROTOCOL: usize = 9;
    pub const CHECKSUM: Field = 10..12;
    pub const SRC_ADDR: Field = 12..16;
    pub const DST_ADDR: Field = 16..20;
}

/// A specification of an IPv4 CIDR block, containing an address and a variable-length
/// subnet masking prefix length.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Cidr {
    address: Address,
    prefix_len: u8, // mask prefix length
}

impl Cidr {
    /// Return the address of this IPv4 CIDR block.
    pub const fn address(&self) -> Address {
        self.address
    }

    /// Create an IPv4 CIDR block from the given address and prefix length.
    ///
    /// # Panics
    /// This function panics if the prefix length is larger than 32.
    pub const fn new(address: Address, prefix_len: u8) -> Cidr {
        assert!(prefix_len <= 32);
        Cidr {
            address,
            prefix_len,
        }
    }
}

pub(crate) trait AddressExt {
    /// Construct an IPv4 address from a sequence of octets, in big-endian.
    ///
    /// # Panics
    /// The function panics if `data` is not four octets long.
    #[allow(unused)]
    fn from_bytes(data: &[u8]) -> Self;

    /// Query whether the address is an unicast address.
    ///
    /// `x_` prefix is to avoid a collision with the still-unstable method in `core::ip`.
    fn x_is_unicast(&self) -> bool;

    /// If `self` is a CIDR-compatible subnet mask, return `Some(prefix_len)`,
    /// where `prefix_len` is the number of leading zeroes. Return `None` otherwise.
    #[allow(unused)]
    fn prefix_len(&self) -> Option<u8>;
}

impl AddressExt for Address {
    fn from_bytes(data: &[u8]) -> Address {
        let mut bytes = [0; ADDR_SIZE];
        bytes.copy_from_slice(data);
        Address::from_bits(u32::from_be_bytes(bytes))
    }

    /// Query whether the address is an unicast address.
    fn x_is_unicast(&self) -> bool {
        !(self.is_broadcast() || self.is_multicast() || self.is_unspecified())
    }

    fn prefix_len(&self) -> Option<u8> {
        let mut ones = true;
        let mut prefix_len = 0;
        for byte in self.octets() {
            let mut mask = 0x80;
            for _ in 0..8 {
                let one = byte & mask != 0;
                if ones {
                    // Expect 1s until first 0
                    if one {
                        prefix_len += 1;
                    } else {
                        ones = false;
                    }
                } else if one {
                    // 1 where 0 was expected
                    return None;
                }
                mask >>= 1;
            }
        }
        Some(prefix_len)
    }
}

impl fmt::Display for Cidr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.address, self.prefix_len)
    }
}

/// A high-level representation of an Internet Protocol version 4 packet header.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Repr {
    pub src_addr: Address,
    pub dst_addr: Address,
    pub next_header: Protocol,
    pub payload_len: usize,
    pub hop_limit: u8,
}

/// A read/write wrapper around an Internet Protocol version 4 packet buffer.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Packet<T: AsRef<[u8]>> {
    buffer: T,
}

impl<T: AsRef<[u8]>> Packet<T> {
    /// Imbue a raw octet buffer with IPv4 packet structure.
    pub const fn new_unchecked(buffer: T) -> Packet<T> {
        Packet { buffer }
    }

    /// Shorthand for a combination of [new_unchecked] and [check_len].
    ///
    /// [new_unchecked]: #method.new_unchecked
    /// [check_len]: #method.check_len
    pub fn new_checked(buffer: T) -> Result<Packet<T>> {
        let packet = Self::new_unchecked(buffer);
        packet.check_len()?;
        Ok(packet)
    }

    /// Ensure that no accessor method will panic if called.
    /// Returns `Err(Error)` if the buffer is too short.
    /// Returns `Err(Error)` if the header length is greater
    /// than total length.
    ///
    /// The result of this check is invalidated by calling [set_header_len]
    /// and [set_total_len].
    ///
    /// [set_header_len]: #method.set_header_len
    /// [set_total_len]: #method.set_total_len
    #[allow(clippy::if_same_then_else)]
    pub fn check_len(&self) -> Result<()> {
        let len = self.buffer.as_ref().len();
        if len < field::DST_ADDR.end {
            Err(Error)
        } else if len < self.header_len() as usize {
            Err(Error)
        } else if self.header_len() as u16 > self.total_len() {
            Err(Error)
        } else if len < self.total_len() as usize {
            Err(Error)
        } else {
            Ok(())
        }
    }

    /// Consume the packet, returning the underlying buffer.
    pub fn into_inner(self) -> T {
        self.buffer
    }

    /// Return the version field.

    pub fn version(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[field::VER_IHL] >> 4
    }

    /// Return the header length, in octets.

    pub fn header_len(&self) -> u8 {
        let data = self.buffer.as_ref();
        (data[field::VER_IHL] & 0x0f) * 4
    }

    /// Return the Differential Services Code Point field.
    pub fn dscp(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[field::DSCP_ECN] >> 2
    }

    /// Return the Explicit Congestion Notification field.
    pub fn ecn(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[field::DSCP_ECN] & 0x03
    }

    /// Return the total length field.

    pub fn total_len(&self) -> u16 {
        let data = self.buffer.as_ref();
        NetworkEndian::read_u16(&data[field::LENGTH])
    }

    /// Return the fragment identification field.

    pub fn ident(&self) -> u16 {
        let data = self.buffer.as_ref();
        NetworkEndian::read_u16(&data[field::IDENT])
    }

    /// Return the "don't fragment" flag.

    pub fn dont_frag(&self) -> bool {
        let data = self.buffer.as_ref();
        NetworkEndian::read_u16(&data[field::FLG_OFF]) & 0x4000 != 0
    }

    /// Return the "more fragments" flag.

    pub fn more_frags(&self) -> bool {
        let data = self.buffer.as_ref();
        NetworkEndian::read_u16(&data[field::FLG_OFF]) & 0x2000 != 0
    }

    /// Return the fragment offset, in octets.

    pub fn frag_offset(&self) -> u16 {
        let data = self.buffer.as_ref();
        NetworkEndian::read_u16(&data[field::FLG_OFF]) << 3
    }

    /// Return the time to live field.

    pub fn hop_limit(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[field::TTL]
    }

    /// Return the next_header (protocol) field.

    pub fn next_header(&self) -> Protocol {
        let data = self.buffer.as_ref();
        Protocol::from(data[field::PROTOCOL])
    }

    /// Return the header checksum field.

    pub fn checksum(&self) -> u16 {
        let data = self.buffer.as_ref();
        NetworkEndian::read_u16(&data[field::CHECKSUM])
    }

    /// Return the source address field.

    pub fn src_addr(&self) -> Address {
        let data = self.buffer.as_ref();
        Address::from_bytes(&data[field::SRC_ADDR])
    }

    /// Return the destination address field.

    pub fn dst_addr(&self) -> Address {
        let data = self.buffer.as_ref();
        Address::from_bytes(&data[field::DST_ADDR])
    }
}
