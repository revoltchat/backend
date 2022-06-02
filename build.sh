#!/bin/bash
# Build base image
docker build -t revolt.chat/base:latest -f Dockerfile .

# Build crates
docker build -t revolt.chat/delta:latest -f crates/delta/Dockerfile .
docker build -t revolt.chat/bonfire:latest -f crates/bonfire/Dockerfile .
