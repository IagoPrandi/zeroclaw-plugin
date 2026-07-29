#!/usr/bin/env sh
set -eu

rustup target add wasm32-wasip2
cargo fetch --locked
cargo test --locked
