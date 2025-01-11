use heapless::LinearMap;

use crate::config::IFACE_NEIGHBOR_CACHE_COUNT;
use crate::time::Instant;
use crate::wire::{HardwareAddress, IpAddress};

/// A neighbor cache backed by a map.
#[derive(Debug)]
#[allow(unused)]
pub struct Cache {
    storage: LinearMap<IpAddress, Neighbor, IFACE_NEIGHBOR_CACHE_COUNT>,
    silent_until: Instant,
}

impl Cache {
    /// Create a cache.
    pub fn new() -> Self {
        Self {
            storage: LinearMap::new(),
            silent_until: Instant::from_millis(0),
        }
    }

    pub(crate) fn flush(&mut self) {
        self.storage.clear()
    }
}

/// A cached neighbor.
///
/// A neighbor mapping translates from a protocol address to a hardware address,
/// and contains the timestamp past which the mapping should be discarded.
#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub struct Neighbor {
    hardware_addr: HardwareAddress,
    expires_at: Instant,
}
