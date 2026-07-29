#!/usr/bin/env sh
set -eu

cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked --release --target wasm32-wasip2
