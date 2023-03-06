default:
    just -l

pwd := `pwd`

run:
    cargo build
    target/debug/cargo-prebuilt

check:
    cargo +nightly fmt --check
    cargo clippy --all-targets --all-features --workspace -- -D warnings

docker:
    docker run -it --rm --pull=always \
    -v {{pwd}}:/prebuilt \
    rust:latest \
    bash
