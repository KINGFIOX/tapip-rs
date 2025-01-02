use tapip_rs::netdev::veth::TapDev;

fn main() {
    let dev = TapDev::new("tap0\0");
    println!("dev: {:?}", dev);
}
