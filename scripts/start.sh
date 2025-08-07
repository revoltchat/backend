#!/usr/bin/env bash
set -e
cargo build \
    --bin revolt-delta \
    --bin revolt-bonfire \
    --bin revolt-autumn \
    --bin revolt-january \
    --bin revolt-gifbox

trap 'pkill -f revolt-' SIGINT
cargo run --bin revolt-delta &
cargo run --bin revolt-bonfire &
cargo run --bin revolt-autumn &
cargo run --bin revolt-january &
cargo run --bin revolt-gifbox
