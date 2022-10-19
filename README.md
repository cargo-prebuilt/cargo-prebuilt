# cargo-prebuilt

[![Rust Build and Test](https://github.com/crow-rest/cargo-prebuilt/actions/workflows/build.yml/badge.svg?event=push)](https://github.com/crow-rest/cargo-prebuilt/actions/workflows/build.yml)
[![Rust Checks](https://github.com/crow-rest/cargo-prebuilt/actions/workflows/checks.yml/badge.svg?event=push)](https://github.com/crow-rest/cargo-prebuilt/actions/workflows/checks.yml)
![Crates.io](https://img.shields.io/crates/v/cargo-prebuilt)

Download prebuilt binaries of some crate.io crates.
<br><br>
Currently cargo-prebuilt targets these platforms for every binary:
- x86_64-unknown-linux-gnu
- x86_64-unknown-linux-musl
- x86_64-apple-darwin
- x86_64-pc-windows-msvc
- aarch64-unknown-linux-gnu
- aarch64-unknown-linux-musl
- aarch64-apple-darwin

## How to Use

To download a crate:
```cargo prebuilt CRATE_NAME```

To download multiple crates:
```cargo prebuilt CRATE_1,CRATE_2,CRATE_3,...```

To download a version of a crate:
```cargo prebuilt CRATE_NAME@VERSION```

To download multiple crates with versions:
```cargo prebuilt CRATE_1@V1,CRATE_2,CRATE_3@V3,...```

## Download

You can download prebuilt binaries [here](https://github.com/crow-rest/cargo-prebuilt-index/releases/tag/cargo-prebuilt-0.1.1).

## Current Limitations

- When using a crate version it must be the extact semver version.
- Does not use *cargo install* as a backup.
- Does not correctly check if a crate is in the main index first.
- Error messages may be confusing or missing.
