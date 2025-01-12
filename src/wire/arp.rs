use super::{Error, Result};
use super::{EthernetAddress, Ipv4Address};
use byteorder::{ByteOrder, NetworkEndian};

pub use super::EthernetProtocol as Protocol;

enum_with_unknown! {
    /// ARP operation type.
    pub enum Operation(u16) {
        Request = 1,
        Reply = 2
    }
}

enum_with_unknown! {
    /// ARP hardware type.
    pub enum Hardware(u16) {
        Ethernet = 1
    }
}

mod field {
    #![allow(non_snake_case)]

    use crate::wire::field::*;

    pub const HTYPE: Field = 0..2;
    pub const PTYPE: Field = 2..4;
    pub const HLEN: usize = 4;
    pub const PLEN: usize = 5;
    pub const OPER: Field = 6..8;

    pub const fn SHA(hardware_len: u8, _protocol_len: u8) -> Field {
        let start = OPER.end;
        start..(start + hardware_len as usize)
    }

    pub const fn SPA(hardware_len: u8, protocol_len: u8) -> Field {
        let start = SHA(hardware_len, protocol_len).end;
        start..(start + protocol_len as usize)
    }

    pub const fn THA(hardware_len: u8, protocol_len: u8) -> Field {
        let start = SPA(hardware_len, protocol_len).end;
        start..(start + hardware_len as usize)
    }

    pub const fn TPA(hardware_len: u8, protocol_len: u8) -> Field {
        let start = THA(hardware_len, protocol_len).end;
        start..(start + protocol_len as usize)
    }
}

/// A high-level representation of an Address Resolution Protocol packet.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub enum Repr {
    /// An Ethernet and IPv4 Address Resolution Protocol packet.
    EthernetIpv4 {
        operation: Operation,
        source_hardware_addr: EthernetAddress,
        source_protocol_addr: Ipv4Address,
        target_hardware_addr: EthernetAddress,
        target_protocol_addr: Ipv4Address,
    },
}

/// A read/write wrapper around an Address Resolution Protocol packet buffer.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Packet<T: AsRef<[u8]>> {
    buffer: T,
}

impl<T: AsRef<[u8]>> Packet<T> {
    /// Imbue a raw octet buffer with ARP packet structure.
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
    ///
    /// The result of this check is invalidated by calling [set_hardware_len] or
    /// [set_protocol_len].
    ///
    /// [set_hardware_len]: #method.set_hardware_len
    /// [set_protocol_len]: #method.set_protocol_len
    #[allow(clippy::if_same_then_else)]
    pub fn check_len(&self) -> Result<()> {
        let len = self.buffer.as_ref().len();
        if len < field::OPER.end {
            Err(Error)
        } else if len < field::TPA(self.hardware_len(), self.protocol_len()).end {
            Err(Error)
        } else {
            Ok(())
        }
    }

    /// Consume the packet, returning the underlying buffer.
    pub fn into_inner(self) -> T {
        self.buffer
    }

    /// Return the hardware type field.
    pub fn hardware_type(&self) -> Hardware {
        let data = self.buffer.as_ref();
        let raw = NetworkEndian::read_u16(&data[field::HTYPE]);
        Hardware::from(raw)
    }

    /// Return the protocol type field.
    pub fn protocol_type(&self) -> Protocol {
        let data = self.buffer.as_ref();
        let raw = NetworkEndian::read_u16(&data[field::PTYPE]);
        Protocol::from(raw)
    }

    /// Return the hardware length field.
    pub fn hardware_len(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[field::HLEN]
    }

    /// Return the protocol length field.
    pub fn protocol_len(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[field::PLEN]
    }

    /// Return the operation field.
    pub fn operation(&self) -> Operation {
        let data = self.buffer.as_ref();
        let raw = NetworkEndian::read_u16(&data[field::OPER]);
        Operation::from(raw)
    }

    /// Return the source hardware address field.
    pub fn source_hardware_addr(&self) -> &[u8] {
        let data = self.buffer.as_ref();
        &data[field::SHA(self.hardware_len(), self.protocol_len())]
    }

    /// Return the source protocol address field.
    pub fn source_protocol_addr(&self) -> &[u8] {
        let data = self.buffer.as_ref();
        &data[field::SPA(self.hardware_len(), self.protocol_len())]
    }

    /// Return the target hardware address field.
    pub fn target_hardware_addr(&self) -> &[u8] {
        let data = self.buffer.as_ref();
        &data[field::THA(self.hardware_len(), self.protocol_len())]
    }

    /// Return the target protocol address field.
    pub fn target_protocol_addr(&self) -> &[u8] {
        let data = self.buffer.as_ref();
        &data[field::TPA(self.hardware_len(), self.protocol_len())]
    }
}
