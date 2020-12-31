# Build Stage
FROM ekidd/rust-musl-builder:nightly-2020-11-19 AS builder
WORKDIR /home/rust/src

RUN USER=root cargo new --bin revolt
WORKDIR ./revolt
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Bundle Stage
FROM scratch
COPY --from=builder /home/rust/src/revolt/target/x86_64-unknown-linux-musl/release/revolt ./
EXPOSE 8000
EXPOSE 9000
CMD ["./revolt"]
