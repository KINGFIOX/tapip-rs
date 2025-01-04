use std::io;

use tapip_rs::netdev::veth::veth_poll;

fn main() -> io::Result<()> {
    veth_poll();
    Ok(())
}
