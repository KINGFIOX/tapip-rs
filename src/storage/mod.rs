mod assembler;
mod packet_buffer;
mod ring_buffer;

pub use self::assembler::Assembler;
pub use self::packet_buffer::{PacketBuffer, PacketMetadata};
pub use self::ring_buffer::RingBuffer;

/// Error returned when enqueuing into a full buffer.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Full;

/// Error returned when dequeuing from an empty buffer.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Empty;

/// A trait for setting a value to a known state.
///
/// In-place analog of Default.
pub trait Resettable {
    fn reset(&mut self);
}
