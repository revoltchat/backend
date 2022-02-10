# Build Stage
#FROM rustlang/rust:nightly-slim AS builder
FROM rustlang/rust:nightly-slim@sha256:7cab2f9ad67980b21258701f8ca9df2b89275a9fed9d1de02bfb35c3d4fd477e AS builder
USER 0:0
WORKDIR /home/rust/src

RUN USER=root cargo new --bin revolt
WORKDIR /home/rust/src/revolt
COPY Cargo.toml Cargo.lock ./
COPY src/bin/dummy.rs ./src/bin/dummy.rs
RUN apt-get update && apt-get install -y libssl-dev pkg-config && cargo build --release --bin dummy

COPY assets/templates ./assets/templates
COPY src ./src
RUN cargo install --locked --path .

# Bundle Stage
FROM debian:buster-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /usr/local/cargo/bin/revolt ./
COPY assets ./assets
EXPOSE 8000
EXPOSE 9000
ENV ROCKET_ADDRESS 0.0.0.0
ENV ROCKET_PORT 8000
CMD ["./revolt"]
