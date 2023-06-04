# Build Stage
FROM --platform="${BUILDPLATFORM}" rust:slim
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
COPY crates/bonfire/Cargo.toml ./crates/bonfire/
COPY crates/delta/Cargo.toml ./crates/delta/
COPY crates/quark/Cargo.toml ./crates/quark/
COPY crates/core/database/Cargo.toml ./crates/core/database/
COPY crates/core/models/Cargo.toml ./crates/core/models/
COPY crates/core/permissions/Cargo.toml ./crates/core/permissions/
COPY crates/core/presence/Cargo.toml ./crates/core/presence/
COPY crates/core/result/Cargo.toml ./crates/core/result/
RUN sh /tmp/build-image-layer.sh deps

# Build all apps
COPY crates ./crates
RUN sh /tmp/build-image-layer.sh apps
