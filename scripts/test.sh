#!/bin/sh

make .make/pyo3-issue-dev

docker run \
    -u $(id -u):$(id -g) \
    -e HOME=/home/$USER \
    -e USER=$USER \
    -e RUST_BACKTRACE=1 \
    --shm-size=2G \
    --mount type=bind,source=$(pwd),target=/pyo3-issue \
    --mount type=bind,source=$(pwd)/.dev-home,target=/home/$USER \
    --mount type=bind,source=$(pwd)/.dev-cargo-cache,target=/opt/.cargo/registry \
    --mount type=bind,source=$(pwd)/.dev-cargo-target,target=/opt/dev-cargo-target \
    --rm -it pyo3-issue-dev:latest \
    /bin/bash -c "cargo test"