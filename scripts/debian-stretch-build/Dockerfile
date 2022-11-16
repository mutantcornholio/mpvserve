FROM debian:stretch

RUN apt-get -yy update && apt-get -yy install git curl bash build-essential pkg-config libssl-dev

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"
ENV CARGO_TARGET_DIR="/cache/cargo/build_target"
ENV PATH="/root/.cargo/bin:${CARGO_HOME}/bin:${RUSTUP_HOME}/bin:$PATH"

RUN rustup install 1.65 && rustup default 1.65

# Force index update
RUN cargo search --limit 0
