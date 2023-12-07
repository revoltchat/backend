# Build Stage
FROM ghcr.io/revoltchat/base:latest AS builder

# Bundle Stage
FROM gcr.io/distroless/cc:nonroot

# copy our compiled binary
COPY --from=builder /home/rust/src/target/release/revolt-delta ./

EXPOSE 8000
ENV ROCKET_ADDRESS 0.0.0.0
ENV ROCKET_PORT 8000

# run as non-privileged user
USER nonroot

# command of container
CMD ["./revolt-delta"]
