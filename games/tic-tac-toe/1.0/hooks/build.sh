#!/bin/bash
set -e

echo "Building tic-tac-toe hooks..."

# Build the WebAssembly module
cargo build --target wasm32-unknown-unknown --release

# Copy the wasm file to the parent directory
cp target/wasm32-unknown-unknown/release/tic_tac_toe_hooks.wasm ../hooks.wasm

echo "Build complete! Output: ../hooks.wasm"