default:
    just -l

pwd := `pwd`

run +ARGS:
    cargo run --no-default-features --features default-rustls -- {{ARGS}}

runr +ARGS:
    cargo run --no-default-features --features default-rustls --release -- {{ARGS}}

runq +ARGS:
    cargo run --no-default-features --features default-rustls --profile=quick -- {{ARGS}}

fmt:
    cargo +nightly fmt

check:
    cargo +nightly fmt --check
    cargo clippy --all-targets --locked --workspace -- -D warnings
    cargo clippy --all-targets --locked --workspace --release -- -D warnings
    cargo deny check

check-nightly:
    cargo +nightly fmt --check
    cargo +nightly clippy --all-targets --locked --workspace -- -D warnings
    cargo +nightly clippy --all-targets --locked --workspace --release -- -D warnings
    cargo +nightly deny check

default_log_level := 'INFO'
runner := 'docker'
sup-lint LOG_LEVEL=default_log_level:
    {{runner}} run \
    -t --rm --pull=always \
    --platform=linux/amd64 \
    -e LOG_LEVEL={{LOG_LEVEL}} \
    -e RUN_LOCAL=true \
    -e SHELL=/bin/bash \
    -e DEFAULT_BRANCH=main \
    -e VALIDATE_ALL_CODEBASE=true \
    -e VALIDATE_JSCPD=false \
    -e VALIDATE_RUST_2015=false \
    -e VALIDATE_RUST_2018=false \
    -e VALIDATE_RUST_2021=false \
    -e VALIDATE_RUST_CLIPPY=false \
    -v ./:/tmp/lint \
    ghcr.io/super-linter/super-linter:slim-latest

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

ink-cross TARGET:
    docker run -it --rm --pull=always \
    -e CARGO_TARGET_DIR=/ptarget \
    --mount type=bind,source={{pwd}},target=/project \
    --mount type=bind,source=$HOME/.cargo/registry,target=/usr/local/cargo/registry \
    ghcr.io/cargo-prebuilt/ink-cross:stable-{{TARGET}} \
    build --verbose --workspace --locked --target {{TARGET}}

ink-crossr TARGET:
    docker run -it --rm --pull=always \
    -e CARGO_TARGET_DIR=/ptarget \
    --mount type=bind,source={{pwd}},target=/project \
    --mount type=bind,source=$HOME/.cargo/registry,target=/usr/local/cargo/registry \
    ghcr.io/cargo-prebuilt/ink-cross:stable-{{TARGET}} \
    build --verbose --workspace --locked --target {{TARGET}} --release

deny:
    docker run -t --rm --pull=always \
    -e CARGO_TARGET_DIR=/ptarget \
    --mount type=bind,source={{pwd}},target=/prebuilt \
    --mount type=bind,source=$HOME/.cargo/registry,target=/usr/local/cargo/registry \
    -w /prebuilt \
    rust:latest \
    bash -c 'cargo run -- cargo-deny && cargo-deny check'

hack:
    docker run -t --rm --pull=always \
    -e CARGO_TARGET_DIR=/ptarget \
    --mount type=bind,source={{pwd}},target=/prebuilt \
    --mount type=bind,source=$HOME/.cargo/registry,target=/usr/local/cargo/registry \
    -w /prebuilt \
    rust:latest \
    bash -c 'cargo run -- cargo-hack && cargo hack check --each-feature --no-dev-deps --verbose --workspace --locked && cargo hack check --feature-powerset --group-features default,default-native,default-rustls,default-no-tls,rustls,rustls-native-certs,native,vendored-openssl --no-dev-deps --verbose --workspace --locked'

msrv:
    docker run -t --rm --pull=always \
    -e CARGO_TARGET_DIR=/ptarget \
    --mount type=bind,source={{pwd}},target=/prebuilt \
    --mount type=bind,source=$HOME/.cargo/registry,target=/usr/local/cargo/registry \
    -w /prebuilt \
    rust:latest \
    bash -c 'cargo install cargo-msrv --version 0.16.0-beta.20 --profile=dev && cargo msrv -- cargo check --verbose --locked'

msrv-verify:
    docker run -t --rm --pull=always \
    -e CARGO_TARGET_DIR=/ptarget \
    --mount type=bind,source={{pwd}},target=/prebuilt \
    --mount type=bind,source=$HOME/.cargo/registry,target=/usr/local/cargo/registry \
    -w /prebuilt \
    rust:latest \
    bash -c 'cargo install cargo-msrv --version 0.16.0-beta.20 --profile=dev && cargo msrv verify -- cargo check --verbose --release --locked'
