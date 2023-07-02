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
    cargo clippy --all-targets --locked --workspace -- -D warnings
    cargo clippy --all-targets --locked --workspace --no-default-features --features default-native -- -D warnings
    cargo clippy --all-targets --locked --workspace --no-default-features --features default-rustls -- -D warnings

docker:
    docker run -it --rm --pull=always \
    -e CARGO_TARGET_DIR=/ptarget \
    --mount type=bind,source={{pwd}},target=/prebuilt \
    --mount type=bind,source=$HOME/.cargo/registry,target=/usr/local/cargo/registry \
    -w /prebuilt \
    rust:latest \
    bash

docker-alpine:
    docker run -it --rm --pull=always \
    -e CARGO_TARGET_DIR=/ptarget \
    --mount type=bind,source={{pwd}},target=/prebuilt \
    --mount type=bind,source=$HOME/.cargo/registry,target=/usr/local/cargo/registry \
    -w /prebuilt \
    rust:alpine \
    sh

hack:
    docker run -t --rm --pull=always \
    -e CARGO_TARGET_DIR=/ptarget \
    --mount type=bind,source={{pwd}},target=/prebuilt \
    --mount type=bind,source=$HOME/.cargo/registry,target=/usr/local/cargo/registry \
    -w /prebuilt \
    rust:latest \
    bash -c 'cargo run -- cargo-hack && cargo hack check --each-feature --no-dev-deps --verbose --workspace --locked && cargo hack check --feature-powerset --group-features rustls,rustls-native-certs,native,vendored-openssl --group-features bright-color,dull-color --no-dev-deps --verbose --workspace --locked'

msrv:
    docker run -t --rm --pull=always \
    -e CARGO_TARGET_DIR=/ptarget \
    --mount type=bind,source={{pwd}},target=/prebuilt \
    --mount type=bind,source=$HOME/.cargo/registry,target=/usr/local/cargo/registry \
    -w /prebuilt \
    rust:latest \
    bash -c 'cargo install cargo-msrv --version 0.16.0-beta.14 --profile=dev && cargo msrv -- cargo check --verbose --locked'

rund +ARGS:
    cargo build
    /ptarget/debug/cargo-prebuilt {{ARGS}}
