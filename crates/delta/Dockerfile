# Build Stage
FROM ghcr.io/revoltchat/base:latest AS builder

# Bundle Stage
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /home/rust/src/target/release/revolt-delta ./

EXPOSE 8000
ENV ROCKET_ADDRESS 0.0.0.0
USER nonroot
CMD ["./revolt-delta"]
