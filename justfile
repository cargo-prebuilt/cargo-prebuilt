default:
    just -l

pwd := `pwd`

check:
    cargo +nightly fmt --check
    cargo clippy --all-targets --all-features --workspace -- -D warnings

docker:
    docker run -it --rm --pull=always \
    -v {{pwd}}:/prebuilt \
    rust:latest \
    bash
