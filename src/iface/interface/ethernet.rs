use super::*;

impl InterfaceInner {
    pub(super) fn process_ethernet<'frame>(
        &mut self,
        sockets: &mut SocketSet,
        meta: crate::phy::PacketMeta,
        frame: &'frame [u8],
        fragments: &'frame mut FragmentsBuffer,
    ) -> Option<EthernetPacket<'frame>> {
        let eth_frame = check!(EthernetFrame::new_checked(frame));

        // Ignore any packets not directed to our hardware address or any of the multicast groups.
        if !eth_frame.dst_addr().is_broadcast()
            && !eth_frame.dst_addr().is_multicast()
            && HardwareAddress::Ethernet(eth_frame.dst_addr()) != self.hardware_addr
        {
            return None;
        }

        match eth_frame.ethertype() {
            EthernetProtocol::Arp => self.process_arp(self.now, &eth_frame),
            EthernetProtocol::Ipv4 => {
                let ipv4_packet = check!(Ipv4Packet::new_checked(eth_frame.payload()));

                self.process_ipv4(
                    sockets,
                    meta,
                    eth_frame.src_addr().into(),
                    &ipv4_packet,
                    fragments,
                )
                .map(EthernetPacket::Ip)
            }
            // Drop all other traffic.
            _ => None,
        }
    }
}
