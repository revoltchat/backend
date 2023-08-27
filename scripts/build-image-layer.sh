#!/bin/sh

if [ -z "$TARGETARCH" ]; then
  :
else
  case "${TARGETARCH}" in
    "amd64")
      LINKER_NAME="x86_64-linux-gnu-gcc"
      LINKER_PACKAGE="gcc-x86-64-linux-gnu"
      BUILD_TARGET="x86_64-unknown-linux-gnu" ;;
    "arm64")
      LINKER_NAME="aarch64-linux-gnu-gcc"
      LINKER_PACKAGE="gcc-aarch64-linux-gnu"
      BUILD_TARGET="aarch64-unknown-linux-gnu" ;;
  esac
fi

tools() {
  apt-get install -y "${LINKER_PACKAGE}"
  rustup target add "${BUILD_TARGET}"
}

deps() {
  mkdir -p \
    crates/bonfire/src \
    crates/delta/src \
    crates/quark/src \
    crates/core/config/src \
    crates/core/database/src \
    crates/core/models/src \
    crates/core/permissions/src \
    crates/core/presence/src \
    crates/core/result/src
  echo 'fn main() { panic!("stub"); }' |
    tee crates/bonfire/src/main.rs |
    tee crates/delta/src/main.rs
  echo '' |
    tee crates/quark/src/lib.rs |
    tee crates/core/config/src/lib.rs |
    tee crates/core/database/src/lib.rs |
    tee crates/core/models/src/lib.rs |
    tee crates/core/permissions/src/lib.rs |
    tee crates/core/presence/src/lib.rs |
    tee crates/core/result/src/lib.rs
  
  if [ -z "$TARGETARCH" ]; then
    cargo build --locked --release
  else
    cargo build --locked --release --target "${BUILD_TARGET}"
  fi
}

apps() {
  touch -am \
    crates/bonfire/src/main.rs \
    crates/delta/src/main.rs \
    crates/quark/src/lib.rs \
    crates/core/config/src/lib.rs \
    crates/core/database/src/lib.rs \
    crates/core/models/src/lib.rs \
    crates/core/permissions/src/lib.rs \
    crates/core/presence/src/lib.rs \
    crates/core/result/src/lib.rs
  
  if [ -z "$TARGETARCH" ]; then
    cargo build --locked --release
  else
    cargo build --locked --release --target "${BUILD_TARGET}"
    mv target _target && mv _target/"${BUILD_TARGET}" target
  fi
}

if [ -z "$TARGETARCH" ]; then
  :
else
  export RUSTFLAGS="-C linker=${LINKER_NAME}"
  export PKG_CONFIG_ALLOW_CROSS="1"
  export PKG_CONFIG_PATH="/usr/lib/pkgconfig:/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/lib/x86_64-linux-gnu/pkgconfig"
fi

"$@"
