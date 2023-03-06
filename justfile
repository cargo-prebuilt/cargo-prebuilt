default:
    just -l

pwd := `pwd`

run ARGS:
    cargo build
    target/debug/cargo-prebuilt {{ARGS}}

check:
    cargo +nightly fmt --check
    cargo clippy --all-targets --all-features --workspace -- -D warnings

docker:
    docker run -it --rm --pull=always \
    -e CARGO_TARGET_DIR=/ptarget \
    -v {{pwd}}:/prebuilt \
    rust:latest \
    bash

rund ARGS:
    cargo build
    /ptarget/debug/cargo-prebuilt {{ARGS}}
