mod utils;

use std::{cmp, collections::HashMap};

use tapip_rs::{
    iface::{Config, Interface, SocketSet},
    phy::Device,
    socket::icmp,
    time::Instant,
    wire::{EthernetAddress, IpAddress, IpCidr, Ipv4Address},
};

fn main() {
    utils::setup_logging("warn");

    let (mut opts, mut free) = utils::create_options();
    // cargo run --example ping -- --tap tap0
    utils::add_tuntap_options(&mut opts, &mut free);
    let mut matches = utils::parse_options(&opts, free);
    let mut device = utils::parse_tuntap_options(&mut matches);
    let mut config = match device.capabilities().medium {
        tapip_rs::phy::Medium::Ethernet => {
            Config::new(EthernetAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x01]).into())
        }
        tapip_rs::phy::Medium::Ip => todo!(),
    };
    config.random_seed = rand::random();
    let mut iface = Interface::new(config, &mut device, Instant::now());
    iface.update_ip_addrs(|ip_addrs| {
        ip_addrs
            .push(IpCidr::new(IpAddress::v4(192, 168, 69, 1), 24))
            .unwrap();
    });
    iface
        .routes_mut()
        .add_default_ipv4_route(Ipv4Address::new(192, 168, 69, 100))
        .unwrap();
    // Create sockets
    let icmp_rx_buffer = icmp::PacketBuffer::new(vec![icmp::PacketMetadata::EMPTY], vec![0; 256]);
    let icmp_tx_buffer = icmp::PacketBuffer::new(vec![icmp::PacketMetadata::EMPTY], vec![0; 256]);
    let icmp_socket = icmp::Socket::new(icmp_rx_buffer, icmp_tx_buffer);
    let mut sockets = SocketSet::new(vec![]);
    let icmp_handle = sockets.add(icmp_socket);

    let mut send_at = Instant::from_millis(0);
    let mut seq_no = 0;
    let mut received = 0;
    let mut echo_payload = [0xffu8; 40];
    // let mut waiting_queue = HashMap::new();
    let ident = 0x22b;

    // loop {
    //     let timestamp = Instant::now();
    //     iface.poll(timestamp, &mut device, &mut sockets);

    //     let timestamp = Instant::now();
    //     let socket = sockets.get_mut::<icmp::Socket>(icmp_handle);
    //     if !socket.is_open() {
    //         socket.bind(icmp::Endpoint::Ident(ident)).unwrap();
    //         send_at = timestamp;
    //     }

    //     if socket.can_send() && seq_no < count as u16 && send_at <= timestamp {
    //         NetworkEndian::write_i64(&mut echo_payload, timestamp.total_millis());

    //         match remote_addr {
    //             IpAddress::Ipv4(_) => {
    //                 let (icmp_repr, mut icmp_packet) = send_icmp_ping!(
    //                     Icmpv4Repr,
    //                     Icmpv4Packet,
    //                     ident,
    //                     seq_no,
    //                     echo_payload,
    //                     socket,
    //                     remote_addr
    //                 );
    //                 icmp_repr.emit(&mut icmp_packet, &device_caps.checksum);
    //             }
    //             IpAddress::Ipv6(address) => {
    //                 let (icmp_repr, mut icmp_packet) = send_icmp_ping!(
    //                     Icmpv6Repr,
    //                     Icmpv6Packet,
    //                     ident,
    //                     seq_no,
    //                     echo_payload,
    //                     socket,
    //                     remote_addr
    //                 );
    //                 icmp_repr.emit(
    //                     &iface.get_source_address_ipv6(&address),
    //                     &address,
    //                     &mut icmp_packet,
    //                     &device_caps.checksum,
    //                 );
    //             }
    //         }

    //         waiting_queue.insert(seq_no, timestamp);
    //         seq_no += 1;
    //         send_at += interval;
    //     }

    //     if socket.can_recv() {
    //         let (payload, _) = socket.recv().unwrap();

    //         match remote_addr {
    //             IpAddress::Ipv4(_) => {
    //                 let icmp_packet = Icmpv4Packet::new_checked(&payload).unwrap();
    //                 let icmp_repr = Icmpv4Repr::parse(&icmp_packet, &device_caps.checksum).unwrap();
    //                 get_icmp_pong!(
    //                     Icmpv4Repr,
    //                     icmp_repr,
    //                     payload,
    //                     waiting_queue,
    //                     remote_addr,
    //                     timestamp,
    //                     received
    //                 );
    //             }
    //             IpAddress::Ipv6(address) => {
    //                 let icmp_packet = Icmpv6Packet::new_checked(&payload).unwrap();
    //                 let icmp_repr = Icmpv6Repr::parse(
    //                     &address,
    //                     &iface.get_source_address_ipv6(&address),
    //                     &icmp_packet,
    //                     &device_caps.checksum,
    //                 )
    //                 .unwrap();
    //                 get_icmp_pong!(
    //                     Icmpv6Repr,
    //                     icmp_repr,
    //                     payload,
    //                     waiting_queue,
    //                     remote_addr,
    //                     timestamp,
    //                     received
    //                 );
    //             }
    //         }
    //     }

    //     waiting_queue.retain(|seq, from| {
    //         if timestamp - *from < timeout {
    //             true
    //         } else {
    //             println!("From {remote_addr} icmp_seq={seq} timeout");
    //             false
    //         }
    //     });

    //     if seq_no == count as u16 && waiting_queue.is_empty() {
    //         break;
    //     }

    //     let timestamp = Instant::now();
    //     match iface.poll_at(timestamp, &sockets) {
    //         Some(poll_at) if timestamp < poll_at => {
    //             let resume_at = cmp::min(poll_at, send_at);
    //             phy_wait(fd, Some(resume_at - timestamp)).expect("wait error");
    //         }
    //         Some(_) => (),
    //         None => {
    //             phy_wait(fd, Some(send_at - timestamp)).expect("wait error");
    //         }
    //     }
    // }
}
