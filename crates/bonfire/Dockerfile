# Build Stage
FROM ghcr.io/revoltchat/base:latest AS builder

# Bundle Stage
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /home/rust/src/target/release/revolt-bonfire ./

EXPOSE 9000
USER nonroot
ENV HOST=0.0.0.0:9000
CMD ["./revolt-bonfire"]
