#!/bin/bash
# Tip: for subsequent builds, don't update Cargo.toml
# in order to not download all the crates again.
# Update Cargo.toml on major release.
version=0.4.1-alpha.4-patch.0
docker build -t revoltchat/server:${version} . &&
    docker push revoltchat/server:${version}
