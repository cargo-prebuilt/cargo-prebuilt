#!/bin/bash

set -euxo pipefail

rustup target add x86_64-pc-windows-msvc
rustup target add aarch64-pc-windows-msvc

if ! cargo-xwin --version &> /dev/null; then
    echo "Installing cargo-xwin..."
    cargo-prebuilt --ci --report-path='/tmp' cargo-xwin
fi

exec cargo "$@"
