use heapless::Vec;

use crate::config::IFACE_MAX_ROUTE_COUNT;
use crate::time::Instant;
use crate::wire::{IpAddress, IpCidr};
use crate::wire::{Ipv4Address, Ipv4Cidr};

const IPV4_DEFAULT: IpCidr = IpCidr::Ipv4(Ipv4Cidr::new(Ipv4Address::new(0, 0, 0, 0), 0));

/// A prefix of addresses that should be routed via a router
#[derive(Debug, Clone, Copy)]
pub struct Route {
    pub cidr: IpCidr,
    pub via_router: IpAddress,
    /// `None` means "forever".
    pub preferred_until: Option<Instant>,
    /// `None` means "forever".
    pub expires_at: Option<Instant>,
}

impl Route {
    pub fn new_ipv4_gateway(gateway: Ipv4Address) -> Route {
        Route {
            cidr: IPV4_DEFAULT,
            via_router: gateway.into(),
            preferred_until: None,
            expires_at: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteTableFull;

impl core::fmt::Display for RouteTableFull {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Route table full")
    }
}

/// A routing table.
#[derive(Debug)]
pub struct Routes {
    storage: Vec<Route, IFACE_MAX_ROUTE_COUNT>,
}

impl Routes {
    /// Creates a new empty routing table.
    pub fn new() -> Self {
        Self {
            storage: Vec::new(),
        }
    }

    pub fn add_default_ipv4_route(
        &mut self,
        gateway: Ipv4Address,
    ) -> Result<Option<Route>, RouteTableFull> {
        let old = self.remove_default_ipv4_route();
        self.storage
            .push(Route::new_ipv4_gateway(gateway))
            .map_err(|_| RouteTableFull)?;
        Ok(old)
    }

    pub fn remove_default_ipv4_route(&mut self) -> Option<Route> {
        if let Some((i, _)) = self
            .storage
            .iter()
            .enumerate()
            .find(|(_, r)| r.cidr == IPV4_DEFAULT)
        {
            Some(self.storage.remove(i))
        } else {
            None
        }
    }
}
