# Build Stage
FROM ghcr.io/revoltchat/base:latest AS builder

# Bundle Stage
FROM gcr.io/distroless/cc:nonroot

# copy our compiled binary
COPY --from=builder /home/rust/src/target/release/revolt-bonfire ./
EXPOSE 9000

# run as non-privileged user
USER nonroot

# command of container
CMD ["./revolt-bonfire"]
