#!/bin/bash

set -euxo pipefail

if ! zig version &> /dev/null; then
    echo "Installing zig..."
    pushd /tmp
    curl -fsSL https://ziglang.org/download/0.11.0/zig-linux-x86_64-0.11.0.tar.xz -o zig.tar.xz
    tar -xvJf zig.tar.xz
    cp zig-linux-x86_64-0.11.0/zig /usr/local/cargo/bin
    cp -r zig-linux-x86_64-0.11.0/lib /usr/local/cargo/bin
    popd
fi

if ! cargo-zigbuild --version &> /dev/null; then
    echo "Installing cargo-zigbuild..."
    cargo-prebuilt --ci --report-path='/tmp' cargo-zigbuild
fi

exec cargo +$RUST_VERSION "$@"
