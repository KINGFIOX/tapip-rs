extern crate cc;

use cc::Build;

fn main() {
    println!("cargo:rerun-if-changed=src/misc/tap.c");
    Build::new().file("src/misc/tap.c").compile("tap");
}
