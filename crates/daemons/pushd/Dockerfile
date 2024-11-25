# Build Stage
FROM ghcr.io/revoltchat/base:latest AS builder

# Bundle Stage
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /home/rust/src/target/release/revolt-pushd ./

USER nonroot
CMD ["./revolt-pushd"]