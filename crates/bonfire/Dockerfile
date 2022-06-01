# Build Stage
FROM rustlang/rust:nightly-slim AS builder
USER 0:0
WORKDIR /home/rust/src

RUN USER=root cargo new --bin bonfire
WORKDIR /home/rust/src/bonfire
RUN apt-get update && apt-get install -y libssl-dev pkg-config

COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo install --locked --path .

# Bundle Stage
FROM debian:buster-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /usr/local/cargo/bin/bonfire ./
EXPOSE 9000
CMD ["./bonfire"]
