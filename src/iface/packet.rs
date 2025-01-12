use crate::wire::*;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, PartialEq)]
pub(crate) enum EthernetPacket<'a> {
    Arp(ArpRepr),
    Ip(Packet<'a>),
}

#[derive(Debug, PartialEq)]
pub(crate) enum Packet<'p> {
    Ipv4(PacketV4<'p>),
}

#[derive(Debug, PartialEq)]
pub(crate) struct PacketV4<'p> {
    header: Ipv4Repr,
    payload: IpPayload<'p>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum IpPayload<'p> {
    Icmpv4(Icmpv4Repr<'p>),
    // Raw(&'p [u8]),
    // Udp(UdpRepr, &'p [u8]),
    // Tcp(TcpRepr<'p>),
}
