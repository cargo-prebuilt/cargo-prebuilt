# cargo-prebuilt

[![Rust Build and Test](https://github.com/cargo-prebuilt/cargo-prebuilt/actions/workflows/build.yml/badge.svg?event=push)](https://github.com/cargo-prebuilt/cargo-prebuilt/actions/workflows/build.yml)
[![Rust Checks](https://github.com/cargo-prebuilt/cargo-prebuilt/actions/workflows/checks.yml/badge.svg?event=push)](https://github.com/cargo-prebuilt/cargo-prebuilt/actions/workflows/checks.yml)
[![Crates.io](https://img.shields.io/crates/v/cargo-prebuilt)](https://crates.io/crates/cargo-prebuilt)
[![rustc-msrv](https://img.shields.io/badge/rustc-1.70%2B-blue?logo=rust)](https://www.rust-lang.org/tools/install)

Download prebuilt binaries of some crate.io crates.

See supported targets, a list of prebuilt crates,
and the official index [here](https://github.com/cargo-prebuilt/index#readme).

_(Some targets may not be prebuilt for some crates)._

Request a crate to be added to the official index [here](https://github.com/cargo-prebuilt/index/issues/new?assignees=&labels=add-crate%2C+under-consideration&template=request-crate.md&title=).

See the currently supported versions [here](docs/SUPPORTED.md)

## How to Use

_Cargo prebuilt overwrites existing binaries by default. To stop this use the
`-s` flag, `--safe` flag, or add `safe = true` to your config file._

To download a crate: `cargo prebuilt CRATE_NAME`

To download multiple crates: `cargo prebuilt CRATE_1,CRATE_2,CRATE_3,...`

To download a version of a crate: `cargo prebuilt CRATE_NAME@VERSION`

To download multiple crates with versions: `cargo prebuilt CRATE_1@V1,CRATE_2,CRATE_3@V3,...`

Need help? Try: `cargo prebuilt --help` or see [Config Info](docs/CONFIG.md)

## Installation

More ways and how to verify your download [here](docs/DOWNLOAD.md).

- You can download the latest prebuilt binaries of cargo-prebuilt
  [here](https://github.com/cargo-prebuilt/cargo-prebuilt/releases/latest).
- Cargo install: `cargo install cargo-prebuilt`
- Cargo prebuilt: `cargo prebuilt cargo-prebuilt`
- Cargo binstall: `cargo binstall cargo-prebuilt --no-confirm`
- Cargo quickinstall: `cargo quickinstall cargo-prebuilt`
- Install script (Unix platforms):

  ```shell
  curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/cargo-prebuilt/cargo-prebuilt/main/scripts/install-cargo-prebuilt.sh \
  -o install-cargo-prebuilt.sh \
  && bash install-cargo-prebuilt.sh \
  && rm install-cargo-prebuilt.sh
  ```

- For GitHub Actions you can use
  [cargo-prebuilt/cargo-prebuilt-action](https://github.com/cargo-prebuilt/cargo-prebuilt-action)

## Building

(Cargo prebuilt requires either the native or rustls feature)

`cargo build`
or for a release version
`cargo build --release`

## Events

To output events use `--out`.

See [Events](docs/EVENTS.md).

## Reports

Reports are generated during crate build time in the index.

They are stored under `$REPORTS/$CRATE/$VERSION` by default.

See [Report Directory ($REPORTS)](docs/PATHS.md#reports).

Use `--report-path` to change where they are stored.

[Report Types](docs/REPORT_TYPES.md)

## Using a custom index

### GitHub public

[Template](https://github.com/cargo-prebuilt/gh-pub-index)
(Usually out of date compared to the main index)

Your URL should be formatted like `github.com/cargo-prebuilt/index`.
cargo-prebuilt requires HTTPS.

- `export PREBUILT_INDEX=gh-pub:$URL`
- `cargo prebuilt --index=gh-pub:$URL CRATES`
- [config.toml](docs/CONFIG.md)

  ```toml
  [key.index]
  index = "gh-pub:$URL"
  pub_key = []
  ```

### GitHub private

**Beta Feature.**

Your URL should be formatted like `github.com/cargo-prebuilt/index`.
Cargo-prebuilt requires HTTPS.

This index requires an auth token with:
Repository permission -> Contents -> Read-only.
[Generate a token](https://github.com/settings/personal-access-tokens/new)

- `export PREBUILT_INDEX=gh-pri:$URL`
- `cargo prebuilt --index=gh-pri:$URL CRATES`
- [config.toml](docs/CONFIG.md)

  ```toml
  [key.index]
  index = "gh-pri:$URL"
  pub_key = []
  auth = ""
  ```
