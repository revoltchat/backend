# Build Stage
FROM ghcr.io/revoltchat/base:latest AS builder
RUN cargo install --locked --path crates/delta

# Bundle Stage
FROM debian:buster-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /usr/local/cargo/bin/revolt-delta ./

EXPOSE 8000
ENV ROCKET_ADDRESS 0.0.0.0
ENV ROCKET_PORT 8000
CMD ["./revolt-delta"]
