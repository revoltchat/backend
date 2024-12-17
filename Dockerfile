# Build Stage
FROM --platform="${BUILDPLATFORM}" rust:1.77.2-slim-bookworm
USER 0:0
WORKDIR /home/rust/src

ARG TARGETARCH

# Install build requirements
RUN dpkg --add-architecture "${TARGETARCH}"
RUN apt-get update && \
    apt-get install -y \
    make \
    pkg-config \
    libssl-dev:"${TARGETARCH}"
COPY scripts/build-image-layer.sh /tmp/
RUN sh /tmp/build-image-layer.sh tools

# Build all dependencies
COPY Cargo.toml Cargo.lock ./
COPY crates/bindings/node/Cargo.toml ./crates/bindings/node/
COPY crates/bonfire/Cargo.toml ./crates/bonfire/
COPY crates/delta/Cargo.toml ./crates/delta/
COPY crates/core/config/Cargo.toml ./crates/core/config/
COPY crates/core/database/Cargo.toml ./crates/core/database/
COPY crates/core/files/Cargo.toml ./crates/core/files/
COPY crates/core/models/Cargo.toml ./crates/core/models/
COPY crates/core/permissions/Cargo.toml ./crates/core/permissions/
COPY crates/core/presence/Cargo.toml ./crates/core/presence/
COPY crates/core/result/Cargo.toml ./crates/core/result/
COPY crates/services/autumn/Cargo.toml ./crates/services/autumn/
COPY crates/services/january/Cargo.toml ./crates/services/january/
COPY crates/daemons/pushd/Cargo.toml ./crates/daemons/pushd/
RUN sh /tmp/build-image-layer.sh deps

# Build all apps
COPY crates ./crates
RUN sh /tmp/build-image-layer.sh apps
