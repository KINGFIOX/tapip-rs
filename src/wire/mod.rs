mod ethernet;

pub use self::ethernet::Frame as EthernetFrame;

mod field {
    pub type Field = ::core::ops::Range<usize>;
    pub type Rest = ::core::ops::RangeFrom<usize>;
}
