# Build Stage
FROM rustlang/rust:nightly-slim AS builder
USER 0:0
WORKDIR /home/rust/src

# Install build requirements
RUN apt-get update && apt-get install -y libssl-dev pkg-config

# Build all crates
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --locked --release
