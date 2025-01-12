/// A TCP sequence number.
///
/// A sequence number is a monotonically advancing integer modulo 2<sup>32</sup>.
/// Sequence numbers do not have a discontiguity when compared pairwise across a signed overflow.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct SeqNumber(pub i32);
