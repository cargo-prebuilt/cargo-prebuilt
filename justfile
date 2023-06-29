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
    cargo clippy --all-targets --locked --workspace --no-default-features --features indexes,rustls,dull-color -- -D warnings

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
    bash -c 'cargo run -- cargo-hack && cargo hack check --each-feature --no-dev-deps --verbose --workspace --locked && cargo hack check --feature-powerset --no-dev-deps --verbose --workspace --locked'

msrv:
    docker run -t --rm --pull=always \
    -e CARGO_TARGET_DIR=/ptarget \
    --mount type=bind,source={{pwd}},target=/prebuilt \
    --mount type=bind,source=$HOME/.cargo/registry,target=/usr/local/cargo/registry \
    -w /prebuilt \
    rust:latest \
    bash -c 'cargo install cargo-msrv --version 0.16.0-beta.14 --profile=dev && cargo msrv -- cargo check --verbose --locked'

check-features:
    cargo build --verbose --workspace --locked # Default
    cargo build --verbose --workspace --locked --no-default-features --features indexes,native # All, native
    cargo build --verbose --workspace --locked --no-default-features --features indexes,vendored-openssl # All, vendored-openssl
    cargo build --verbose --workspace --locked --no-default-features --features indexes,rustls # All, rustls
    cargo build --verbose --workspace --locked --no-default-features --features indexes,rustls-native-certs # All, rustls native certs

rund +ARGS:
    cargo build
    /ptarget/debug/cargo-prebuilt {{ARGS}}
