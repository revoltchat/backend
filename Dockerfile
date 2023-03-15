# Build Stage
FROM --platform="${BUILDPLATFORM}" rustlang/rust:nightly-slim AS builder
USER 0:0
WORKDIR /home/rust/src

# Install build requirements

ARG TARGETARCH

RUN dpkg --add-architecture "${TARGETARCH}"
RUN apt-get update && \
    apt-get install -y \
    make \
    pkg-config \
    libssl-dev:"${TARGETARCH}"

# Build all crates
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN \
  case "${TARGETARCH}" in \
    "amd64") \
      LINKER_NAME="x86_64-linux-gnu-gcc"; \
      LINKER_PACKAGE="gcc-x86-64-linux-gnu"; \
      BUILD_TARGET="x86_64-unknown-linux-gnu";; \
    "arm64") \
      LINKER_NAME="aarch64-linux-gnu-gcc"; \
      LINKER_PACKAGE="gcc-aarch64-linux-gnu"; \
      BUILD_TARGET="aarch64-unknown-linux-gnu";; \
  esac; \
  \
  apt-get install -y "${LINKER_PACKAGE}" && rustup target add "${BUILD_TARGET}" && \
  \
  RUSTFLAGS="-C linker=${LINKER_NAME}" \
  PKG_CONFIG_ALLOW_CROSS="1" \
  PKG_CONFIG_PATH="/usr/lib/pkgconfig:/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/lib/x86_64-linux-gnu/pkgconfig" \
    cargo build --locked --release --target "${BUILD_TARGET}" && \
    mv target _target && mv _target/"${BUILD_TARGET}" target
