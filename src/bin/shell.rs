use std::io;

use tapip_rs::netdev::veth::veth_poll;

fn main() -> io::Result<()> {
    env_logger::init();
    veth_poll();
    Ok(())
}
