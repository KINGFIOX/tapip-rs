use heapless::Vec;

use crate::config::IFACE_MAX_ADDR_COUNT;
use crate::phy::{Device, DeviceCapabilities, Medium, PacketMeta, RxToken};
use crate::rand::Rand;
use crate::time::Instant;
use crate::wire::*;

use super::fragmentation::{Fragmenter, FragmentsBuffer};
use super::neighbor::Cache as NeighborCache;
use super::packet::*;
use super::route::Routes;
use super::SocketSet;

mod ethernet;
mod ipv4;

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

/// Result returned by [`Interface::poll`].
///
/// This contains information on whether socket states might have changed.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PollResult {
    /// Socket state is guaranteed to not have changed.
    None,
    /// You should check the state of sockets again for received data or completion of operations.
    SocketStateChanged,
}

/// Result returned by [`Interface::poll_ingress_single`].
///
/// This contains information on whether a packet was processed or not,
/// and whether it might've affected socket states.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PollIngressSingleResult {
    /// No packet was processed. You don't need to call [`Interface::poll_ingress_single`]
    /// again, until more packets arrive.
    ///
    /// Socket state is guaranteed to not have changed.
    None,
    /// A packet was processed.
    ///
    /// There may be more packets in the device's RX queue, so you should call [`Interface::poll_ingress_single`] again.
    ///
    /// Socket state is guaranteed to not have changed.
    PacketProcessed,
    /// A packet was processed, which might have caused socket state to change.
    ///
    /// There may be more packets in the device's RX queue, so you should call [`Interface::poll_ingress_single`] again.
    ///
    /// You should check the state of sockets again for received data or completion of operations.
    SocketStateChanged,
}

impl Interface {
    /// Transmit packets queued in the sockets, and receive packets queued
    /// in the device.
    ///
    /// This function returns a value indicating whether the state of any socket
    /// might have changed.
    ///
    /// ## DoS warning
    ///
    /// This function processes all packets in the device's queue. This can
    /// be an unbounded amount of work if packets arrive faster than they're
    /// processed.
    ///
    /// If this is a concern for your application (i.e. your environment doesn't
    /// have preemptive scheduling, or `poll()` is called from a main loop where
    /// other important things are processed), you may use the lower-level methods
    /// [`poll_egress()`](Self::poll_egress) and [`poll_ingress_single()`](Self::poll_ingress_single).
    /// This allows you to insert yields or process other events between processing
    /// individual ingress packets.
    pub fn poll(
        &mut self,
        timestamp: Instant,
        device: &mut (impl Device + ?Sized),
        sockets: &mut SocketSet<'_>,
    ) -> PollResult {
        self.inner.now = timestamp;

        let mut res = PollResult::None;

        // Process ingress while there's packets available.
        loop {
            match self.socket_ingress(device, sockets) {
                PollIngressSingleResult::None => break,
                PollIngressSingleResult::PacketProcessed => {}
                PollIngressSingleResult::SocketStateChanged => res = PollResult::SocketStateChanged,
            }
        }

        // Process egress.
        match self.poll_egress(timestamp, device, sockets) {
            PollResult::None => {}
            PollResult::SocketStateChanged => res = PollResult::SocketStateChanged,
        }

        res
    }

    /// Transmit packets queued in the sockets.
    ///
    /// This function returns a value indicating whether the state of any socket
    /// might have changed.
    ///
    /// This is guaranteed to always perform a bounded amount of work.
    pub fn poll_egress(
        &mut self,
        timestamp: Instant,
        device: &mut (impl Device + ?Sized),
        sockets: &mut SocketSet<'_>,
    ) -> PollResult {
        self.inner.now = timestamp;

        match self.inner.caps.medium {
            _ => {}
        }

        self.socket_egress(device, sockets)
    }

    fn socket_egress(
        &mut self,
        device: &mut (impl Device + ?Sized),
        sockets: &mut SocketSet<'_>,
    ) -> PollResult {
        // let _caps = device.capabilities();

        // enum EgressError {
        //     Exhausted,
        //     Dispatch,
        // }

        // let mut result = PollResult::None;
        // for item in sockets.items_mut() {
        //     if !item
        //         .meta
        //         .egress_permitted(self.inner.now, |ip_addr| self.inner.has_neighbor(&ip_addr))
        //     {
        //         continue;
        //     }

        //     let mut neighbor_addr = None;
        //     let mut respond = |inner: &mut InterfaceInner, meta: PacketMeta, response: Packet| {
        //         neighbor_addr = Some(response.ip_repr().dst_addr());
        //         let t = device.transmit(inner.now).ok_or_else(|| {
        //             net_debug!("failed to transmit IP: device exhausted");
        //             EgressError::Exhausted
        //         })?;

        //         inner
        //             .dispatch_ip(t, meta, response, &mut self.fragmenter)
        //             .map_err(|_| EgressError::Dispatch)?;

        //         result = PollResult::SocketStateChanged;

        //         Ok(())
        //     };

        //     let result = match &mut item.socket {
        //         Socket::Raw(socket) => socket.dispatch(&mut self.inner, |inner, (ip, raw)| {
        //             respond(
        //                 inner,
        //                 PacketMeta::default(),
        //                 Packet::new(ip, IpPayload::Raw(raw)),
        //             )
        //         }),
        //         Socket::Icmp(socket) => {
        //             socket.dispatch(&mut self.inner, |inner, response| match response {
        //                 (IpRepr::Ipv4(ipv4_repr), IcmpRepr::Ipv4(icmpv4_repr)) => respond(
        //                     inner,
        //                     PacketMeta::default(),
        //                     Packet::new_ipv4(ipv4_repr, IpPayload::Icmpv4(icmpv4_repr)),
        //                 ),
        //                 #[allow(unreachable_patterns)]
        //                 _ => unreachable!(),
        //             })
        //         }
        //         Socket::Udp(socket) => {
        //             socket.dispatch(&mut self.inner, |inner, meta, (ip, udp, payload)| {
        //                 respond(inner, meta, Packet::new(ip, IpPayload::Udp(udp, payload)))
        //             })
        //         }
        //         Socket::Tcp(socket) => socket.dispatch(&mut self.inner, |inner, (ip, tcp)| {
        //             respond(
        //                 inner,
        //                 PacketMeta::default(),
        //                 Packet::new(ip, IpPayload::Tcp(tcp)),
        //             )
        //         }),
        //     };

        //     match result {
        //         Err(EgressError::Exhausted) => break, // Device buffer full.
        //         Err(EgressError::Dispatch) => {
        //             // `NeighborCache` already takes care of rate limiting the neighbor discovery
        //             // requests from the socket. However, without an additional rate limiting
        //             // mechanism, we would spin on every socket that has yet to discover its
        //             // neighbor.
        //             item.meta.neighbor_missing(
        //                 self.inner.now,
        //                 neighbor_addr.expect("non-IP response packet"),
        //             );
        //         }
        //         Ok(()) => {}
        //     }
        // }
        // result
        todo!()
    }

    fn socket_ingress(
        &mut self,
        device: &mut (impl Device + ?Sized),
        sockets: &mut SocketSet<'_>,
    ) -> PollIngressSingleResult {
        // let Some((rx_token, tx_token)) = device.receive(self.inner.now) else {
        //     return PollIngressSingleResult::None;
        // };

        // let rx_meta = rx_token.meta();
        // rx_token.consume(|frame| {
        //     if frame.is_empty() {
        //         return PollIngressSingleResult::PacketProcessed;
        //     }

        //     match self.inner.caps.medium {
        //         Medium::Ethernet => {
        //             if let Some(packet) =
        //                 self.inner
        //                     .process_ethernet(sockets, rx_meta, frame, &mut self.fragments)
        //             {
        //                 if let Err(err) =
        //                     self.inner.dispatch(tx_token, packet, &mut self.fragmenter)
        //                 {
        //                     net_debug!("Failed to send response: {:?}", err);
        //                 }
        //             }
        //         }
        //         Medium::Ip => {
        //             if let Some(packet) =
        //                 self.inner
        //                     .process_ip(sockets, rx_meta, frame, &mut self.fragments)
        //             {
        //                 if let Err(err) = self.inner.dispatch_ip(
        //                     tx_token,
        //                     PacketMeta::default(),
        //                     packet,
        //                     &mut self.fragmenter,
        //                 ) {
        //                     net_debug!("Failed to send response: {:?}", err);
        //                 }
        //             }
        //         }
        //     }

        //     // TODO: Propagate the PollIngressSingleResult from deeper.
        //     // There's many received packets that we process but can't cause sockets
        //     // to change state. For example IP fragments, multicast stuff, ICMP pings
        //     // if they dont't match any raw socket...
        //     // We should return `PacketProcessed` for these to save the user from
        //     // doing useless socket polls.
        //     PollIngressSingleResult::SocketStateChanged
        // })
        todo!()
    }
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

macro_rules! check {
    ($e:expr) => {
        match $e {
            Ok(x) => x,
            Err(_) => {
                // concat!/stringify! doesn't work with defmt macros
                #[cfg(not(feature = "defmt"))]
                net_trace!(concat!("iface: malformed ", stringify!($e)));
                #[cfg(feature = "defmt")]
                net_trace!("iface: malformed");
                return Default::default();
            }
        }
    };
}
use check;
