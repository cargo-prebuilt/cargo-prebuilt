default:
    just -l

pwd := `pwd`

run +ARGS:
    cargo build
    target/debug/cargo-prebuilt {{ARGS}}

fmt:
    cargo +nightly fmt

check:
    cargo +nightly fmt --check
    cargo clippy --all-targets --all-features --workspace -- -D warnings

docker:
    docker run -it --rm --pull=always \
    -e CARGO_TARGET_DIR=/ptarget \
    --mount type=bind,source={{pwd}},target=/prebuilt \
    --mount type=bind,source=$HOME/.cargo/registry,target=/usr/local/cargo/registry \
    -w /prebuilt \
    rust:latest \
    bash

msrv:
    docker run -t --rm --pull=always \
    -e CARGO_TARGET_DIR=/ptarget \
    --mount type=bind,source={{pwd}},target=/prebuilt \
    --mount type=bind,source=$HOME/.cargo/registry,target=/usr/local/cargo/registry \
    -w /prebuilt \
    rust:latest \
    bash -c 'cargo install cargo-msrv && cargo msrv'

rund +ARGS:
    cargo build
    /ptarget/debug/cargo-prebuilt {{ARGS}}
