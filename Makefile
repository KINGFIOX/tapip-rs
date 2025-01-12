cargo:
	cargo build --example ping

gdb: cargo
	gdb -ex "source ./.gdbinit" --args target/debug/examples/ping --tap tap0 10.249.12.73
