#!/bin/bash
cargo build
./target/debug/shell &
pid=$!
ip addr add 192.168.0.1/24 dev tun0
ip link set up dev tun0
trap "kill $pid" INT TERM
wait $pid
