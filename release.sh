#!/bin/bash

mkdir target/bundle

set -e

echo X86_64 Windows
cargo build --release --target x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/fun_with_squares.exe target/bundle

echo X86_64 Linux
cargo build --release --target x86_64-unknown-linux-gnu
cp target/x86_64-unknown-linux-gnu/release/fun_with_squares target/bundle/fun_with_squares.linux.x86_64

echo ARM Linux
cargo build --release --target aarch64-unknown-linux-gnu
cp target/aarch64-unknown-linux-gnu/release/fun_with_squares target/bundle/fun_with_squares.linux.aarch64

# echo X86_64 Mac
# cargo build --release --target x86_64-apple-darwin

# echo Apple Silicon
# cargo build --release --target aarch64-apple-darwin
