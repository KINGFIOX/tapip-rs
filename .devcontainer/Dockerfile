FROM ubuntu:latest

RUN apt-get update -y \
    && apt-get install -y rustup git build-essential

RUN rustup default stable

RUN apt-get install -y iproute2 iputils-ping tcpdump net-tools