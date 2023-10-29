FROM ubuntu:20.04

RUN ln -fs /usr/share/zoneinfo/UTC /etc/localtime
RUN apt-get update && apt-get -yy install git curl bash build-essential libssl-dev pkg-config

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o rustup.sh
RUN sh rustup.sh -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup install 1.65 && rustup default 1.65
# Force index update
RUN cargo search --limit 0
