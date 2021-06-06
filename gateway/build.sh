#!/bin/bash
set -e

RUSTFLAGS='-C link-arg=-s' cargo +nightly build --target wasm32-unknown-unknown --release  -Z avoid-dev-deps
cp ../target/wasm32-unknown-unknown/release/gateway.wasm ../res/
