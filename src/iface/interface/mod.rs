use heapless::Vec;

use crate::config::IFACE_MAX_ADDR_COUNT;
use crate::phy::{Device, DeviceCapabilities};
use crate::rand::Rand;
use crate::time::Instant;
use crate::wire::*;

use super::fragmentation::{Fragmenter, FragmentsBuffer};
use super::neighbor::Cache as NeighborCache;
use super::route::Routes;

/// Configuration structure used for creating a network interface.
#[non_exhaustive]
pub struct Config {
    /// Random seed.
    ///
    /// It is strongly recommended that the random seed is different on each boot,
    /// to avoid problems with TCP port/sequence collisions.
    ///
    /// The seed doesn't have to be cryptographically secure.
    pub random_seed: u64,

    /// Set the Hardware address the interface will use.
    ///
    /// # Panics
    /// Creating the interface panics if the address is not unicast.
    pub hardware_addr: HardwareAddress,
}

impl Config {
    pub fn new(hardware_addr: HardwareAddress) -> Self {
        Config {
            random_seed: 0,
            hardware_addr,
        }
    }
}

/// A  network interface.
///
/// The network interface logically owns a number of other data structures; to avoid
/// a dependency on heap allocation, it instead owns a `BorrowMut<[T]>`, which can be
/// a `&mut [T]`, or `Vec<T>` if a heap is available.
#[allow(unused)]
pub struct Interface {
    pub(crate) inner: InterfaceInner,
    fragments: FragmentsBuffer,
    fragmenter: Fragmenter,
}

/// The device independent part of an Ethernet network interface.
///
/// Separating the device from the data required for processing and dispatching makes
/// it possible to borrow them independently. For example, the tx and rx tokens borrow
/// the `device` mutably until they're used, which makes it impossible to call other
/// methods on the `Interface` in this time (since its `device` field is borrowed
/// exclusively). However, it is still possible to call methods on its `inner` field.
#[allow(unused)]
pub struct InterfaceInner {
    caps: DeviceCapabilities,
    now: Instant,
    rand: Rand,

    neighbor_cache: NeighborCache,
    hardware_addr: HardwareAddress,
    ip_addrs: Vec<IpCidr, IFACE_MAX_ADDR_COUNT>,
    any_ip: bool,
    routes: Routes,
}

/// setter
impl Interface {
    pub fn routes_mut(&mut self) -> &mut Routes {
        &mut self.inner.routes
    }
}

impl Interface {
    pub fn new(config: Config, device: &mut (impl Device + ?Sized), now: Instant) -> Self {
        let caps = device.capabilities();
        assert_eq!(
            config.hardware_addr.medium(),
            caps.medium,
            "The hardware address does not match the medium of the interface."
        );

        let mut rand = Rand::new(config.random_seed);

        let mut ipv4_id;

        loop {
            ipv4_id = rand.rand_u16();
            if ipv4_id != 0 {
                break;
            }
        }

        Interface {
            fragments: FragmentsBuffer {},
            fragmenter: Fragmenter::new(),
            inner: InterfaceInner {
                now,
                caps,
                hardware_addr: config.hardware_addr,
                ip_addrs: Vec::new(),
                any_ip: false,
                routes: Routes::new(),
                neighbor_cache: NeighborCache::new(),
                rand,
            },
        }
    }

    /// Update the IP addresses of the interface.
    ///
    /// # Panics
    /// This function panics if any of the addresses are not unicast.
    pub fn update_ip_addrs<F: FnOnce(&mut Vec<IpCidr, IFACE_MAX_ADDR_COUNT>)>(&mut self, f: F) {
        f(&mut self.inner.ip_addrs);
        InterfaceInner::flush_neighbor_cache(&mut self.inner);
        InterfaceInner::check_ip_addrs(&self.inner.ip_addrs);
    }
}

impl InterfaceInner {
    fn flush_neighbor_cache(&mut self) {
        self.neighbor_cache.flush()
    }

    fn check_ip_addrs(addrs: &[IpCidr]) {
        for cidr in addrs {
            if !cidr.address().is_unicast() && !cidr.address().is_unspecified() {
                panic!("IP address {} is not unicast", cidr.address())
            }
        }
    }
}
