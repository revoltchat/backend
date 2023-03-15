# Build Stage
FROM --platform="${TARGETPLATFORM}" revoltchat/base:latest AS base

# Bundle Stage
FROM debian:buster-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=base /home/rust/src/target/release/revolt-bonfire ./
EXPOSE 9000
CMD ["./revolt-bonfire"]
