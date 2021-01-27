# Build Stage
FROM ekidd/rust-musl-builder:nightly-2021-01-01 AS builder
WORKDIR /home/rust/src

RUN USER=root cargo new --bin revolt
WORKDIR ./revolt
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Bundle Stage
FROM alpine:latest
RUN apk update && apk add ca-certificates && rm -rf /var/cache/apk/*
COPY --from=builder /home/rust/src/revolt/target/x86_64-unknown-linux-musl/release/revolt ./
EXPOSE 8000
EXPOSE 9000
ENV ROCKET_ADDRESS 0.0.0.0
ENV ROCKET_PORT 8000
CMD ["./revolt"]
