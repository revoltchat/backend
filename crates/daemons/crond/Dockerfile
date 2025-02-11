# Build Stage
FROM ghcr.io/revoltchat/base:latest AS builder
FROM debian:12 AS debian

# Bundle Stage
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /home/rust/src/target/release/revolt-crond ./

USER nonroot
CMD ["./revolt-crond"]