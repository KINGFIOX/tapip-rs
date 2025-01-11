#[macro_use]
mod macros; // this should be previous than the mod phy; fuck rust
mod rand;

pub mod iface;
pub mod phy;
pub mod time;
pub mod wire;

pub mod config {
    pub const ASSEMBLER_MAX_SEGMENT_COUNT: usize = 4;
    pub const DNS_MAX_NAME_SIZE: usize = 255;
    pub const DNS_MAX_RESULT_COUNT: usize = 1;
    pub const DNS_MAX_SERVER_COUNT: usize = 1;
    pub const FRAGMENTATION_BUFFER_SIZE: usize = 4096;
    pub const IFACE_MAX_ADDR_COUNT: usize = 8;
    pub const IFACE_MAX_MULTICAST_GROUP_COUNT: usize = 4;
    pub const IFACE_MAX_ROUTE_COUNT: usize = 4;
    pub const IFACE_MAX_SIXLOWPAN_ADDRESS_CONTEXT_COUNT: usize = 4;
    pub const IFACE_NEIGHBOR_CACHE_COUNT: usize = 3;
    pub const REASSEMBLY_BUFFER_COUNT: usize = 4;
    pub const REASSEMBLY_BUFFER_SIZE: usize = 1500;
    pub const RPL_RELATIONS_BUFFER_COUNT: usize = 16;
    pub const RPL_PARENTS_BUFFER_COUNT: usize = 8;
    pub const IPV6_HBH_MAX_OPTIONS: usize = 4;
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
