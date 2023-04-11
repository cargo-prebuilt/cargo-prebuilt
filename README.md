# cargo-prebuilt

[![Rust Build and Test](https://github.com/cargo-prebuilt/cargo-prebuilt/actions/workflows/build.yml/badge.svg?event=push)](https://github.com/cargo-prebuilt/cargo-prebuilt/actions/workflows/build.yml)
[![Rust Checks](https://github.com/cargo-prebuilt/cargo-prebuilt/actions/workflows/checks.yml/badge.svg?event=push)](https://github.com/cargo-prebuilt/cargo-prebuilt/actions/workflows/checks.yml)
[![Crates.io](https://img.shields.io/crates/v/cargo-prebuilt)](https://crates.io/crates/cargo-prebuilt)
[![rustc-msrv](https://img.shields.io/badge/rustc-1.60%2B-blue?logo=rust)](https://www.rust-lang.org/tools/install)

Download prebuilt binaries of some crate.io crates.

See supported targets and a list of prebuilt crates [here](https://github.com/cargo-prebuilt/index#readme).

*(Some targets may not be prebuilt for some crates)*

Request a crate to be added [here](https://github.com/cargo-prebuilt/index/issues/new?assignees=&labels=add-crate%2C+under-consideration&template=request-crate.md&title=).

## How to Use

To download a crate: ```cargo prebuilt CRATE_NAME```

To download multiple crates: ```cargo prebuilt CRATE_1,CRATE_2,CRATE_3,...```

To download a version of a crate: ```cargo prebuilt CRATE_NAME@VERSION```

To download multiple crates with versions: ```cargo prebuilt CRATE_1@V1,CRATE_2,CRATE_3@V3,...```

Need help? Try: ```cargo prebuilt --help```

## Installation

- You can download the latest prebuilt binaries of cargo-prebuilt [here](https://github.com/cargo-prebuilt/cargo-prebuilt/releases/latest).
- Cargo install: ```cargo install cargo-prebuilt``` or ```cargo install cargo-prebuilt --profile=quick-build```
- Cargo binstall ```cargo binstall cargo-prebuilt --no-confirm```

## Reports

Reports are generated when a crate is built in the index.

They are stored under INSTALL_PATH/.prebuilt/reports/CRATE/VERSION.

Report types (--reports):
- license-out: Print license to stdout.
- license-dl: Download license and put it under the prebuilt folder in the installation dir. (Default on)
- deps-out: Print deps tree to stdout.
- deps-dl: Download deps tree and put it under the prebuilt folder in the installation dir.
- audit-out: Print audit to stdout.
- audit-dl: Download audit and put it under the prebuilt folder in the installation dir.

## Using a custom index

#### GitHub public

[Template](https://github.com/cargo-prebuilt/gh-pub-index)

Your url should be formatted like ```github.com/cargo-prebuilt/index```. cargo-prebuilt can only use https indexes.

- ```export PREBUILT_INDEX=gh-pub:URL```
- ```cargo prebuilt --index=gh-pub:URL CRATES```

#### GitHub private

Under development
