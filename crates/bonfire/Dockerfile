# Build Stage
FROM ghcr.io/revoltchat/base:latest AS builder
RUN cargo install --locked --path crates/bonfire

# Bundle Stage
FROM debian:buster-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /usr/local/cargo/bin/revolt-bonfire ./
EXPOSE 9000
CMD ["./revolt-bonfire"]
