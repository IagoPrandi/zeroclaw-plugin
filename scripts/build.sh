#!/usr/bin/env sh
set -eu

cargo build --locked --release --target wasm32-wasip2
sha256sum target/wasm32-wasip2/release/solana_transaction_guardian.wasm
