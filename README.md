# cargo-prebuilt

[![Rust Build and Test](https://github.com/cargo-prebuilt/cargo-prebuilt/actions/workflows/build.yml/badge.svg?event=push)](https://github.com/cargo-prebuilt/cargo-prebuilt/actions/workflows/build.yml)
[![Rust Checks](https://github.com/cargo-prebuilt/cargo-prebuilt/actions/workflows/checks.yml/badge.svg?event=push)](https://github.com/cargo-prebuilt/cargo-prebuilt/actions/workflows/checks.yml)
[![Crates.io](https://img.shields.io/crates/v/cargo-prebuilt)](https://crates.io/crates/cargo-prebuilt)
[![rustc-msrv](https://img.shields.io/badge/rustc-1.64%2B-blue?logo=rust)](https://www.rust-lang.org/tools/install)

Download prebuilt binaries of some crate.io crates.

See supported targets and a list of prebuilt crates [here](https://github.com/cargo-prebuilt/index#readme).

*(Some targets may not be prebuilt for some crates)*

Request a crate to be added [here](https://github.com/cargo-prebuilt/index/issues/new?assignees=&labels=add-crate%2C+under-consideration&template=request-crate.md&title=).

## How to Use

To download a crate: ```cargo prebuilt CRATE_NAME```

To download multiple crates: ```cargo prebuilt CRATE_1,CRATE_2,CRATE_3,...```

To download a version of a crate: ```cargo prebuilt CRATE_NAME@VERSION```

To download multiple crates with versions: ```cargo prebuilt CRATE_1@V1,CRATE_2,CRATE_3@V3,...```

Need help? Try: ```cargo prebuilt --help``` or see [Config Info](CONFIG.md)

## Installation

- You can download the latest prebuilt binaries of cargo-prebuilt [here](https://github.com/cargo-prebuilt/cargo-prebuilt/releases/latest).
- Cargo install: ```cargo install cargo-prebuilt``` or ```cargo install cargo-prebuilt --profile=quick-build```
- Cargo binstall: ```cargo binstall cargo-prebuilt --no-confirm```
- Homebrew: ```brew install crow-rest/harmless/cargo-prebuilt```
- For github actions you can use [cargo-prebuilt/cargo-prebuilt-action](https://github.com/cargo-prebuilt/cargo-prebuilt-action)

## Building

#### vendored-openssl (default)
```cargo install cargo-prebuilt --no-default-features --features indexes,hashes,bright-color,vendored-openssl```

#### native tls (github actions/releases default)
```cargo install cargo-prebuilt --no-default-features --features indexes,hashes,bright-color,native```

#### rustls
```cargo install cargo-prebuilt --no-default-features --features indexes,hashes,bright-color,rustls```

#### rustls with native certs
```cargo install cargo-prebuilt --no-default-features --features indexes,hashes,bright-color,rustls-native-certs```

#### limit indexes used
Remove ```indexes``` feature included by default, then add the features you want below:
- [github-public](#github-public)
- [github-private](#github-private) (Not supported yet)
- [gitlab-public](#gitlab-public) (Not supported yet)
- [gitlab-private](#gitlab-private) (Not supported yet)
- [forgejo-public](#forgejo-public) (Not supported yet)
- [forgejo-private](#forgejo-private) (Not supported yet)
- [gitea-public](#gitea-public) (Not supported yet)
- [gitea-private](#gitea-private) (Not supported yet)

#### colors
```bright-color``` feature is used by default.
Add feature ```bright-color``` or ```dull-color```.

## Reports

Reports are generated when a crate is built in the index.

They are stored under $HOME/.prebuilt/reports/CRATE/VERSION bu default.

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

Your url should be formatted like ```github.com/cargo-prebuilt/index```. cargo-prebuilt requires https.

- ```export PREBUILT_INDEX=gh-pub:$URL```
- ```cargo prebuilt --index=gh-pub:$URL CRATES```
- [config.toml](CONFIG.md) ```index = "gh-pub:$URL"```

#### GitHub private

Under development
