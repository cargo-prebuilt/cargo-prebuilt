# cargo-prebuilt

[![Rust Build and Test](https://github.com/cargo-prebuilt/cargo-prebuilt/actions/workflows/build.yml/badge.svg?event=push)](https://github.com/cargo-prebuilt/cargo-prebuilt/actions/workflows/build.yml)
[![Rust Checks](https://github.com/cargo-prebuilt/cargo-prebuilt/actions/workflows/checks.yml/badge.svg?event=push)](https://github.com/cargo-prebuilt/cargo-prebuilt/actions/workflows/checks.yml)
[![Crates.io](https://img.shields.io/crates/v/cargo-prebuilt)](https://crates.io/crates/cargo-prebuilt)
[![rustc-msrv](https://img.shields.io/badge/rustc-1.66%2B-blue?logo=rust)](https://www.rust-lang.org/tools/install)

Download prebuilt binaries of some crate.io crates.

See supported targets and a list of prebuilt crates [here](https://github.com/cargo-prebuilt/index#readme).

*(Some targets may not be prebuilt for some crates)*

Request a crate to be added [here](https://github.com/cargo-prebuilt/index/issues/new?assignees=&labels=add-crate%2C+under-consideration&template=request-crate.md&title=).

## How to Use

*Cargo prebuilt overwrites existing binaries by default. To stop this use the ```-s``` flag, ```--safe``` flag, or add ```safe = true``` to your config file.*

To download a crate: ```cargo prebuilt CRATE_NAME```

To download multiple crates: ```cargo prebuilt CRATE_1,CRATE_2,CRATE_3,...```

To download a version of a crate: ```cargo prebuilt CRATE_NAME@VERSION```

To download multiple crates with versions: ```cargo prebuilt CRATE_1@V1,CRATE_2,CRATE_3@V3,...```

Need help? Try: ```cargo prebuilt --help``` or see [Config Info](docs/CONFIG.md)

## Installation

- You can download the latest prebuilt binaries of cargo-prebuilt [here](https://github.com/cargo-prebuilt/cargo-prebuilt/releases/latest).
- Cargo install: ```cargo install cargo-prebuilt``` or ```cargo install cargo-prebuilt --profile=quick-build```
- Cargo prebuilt: ```cargo prebuilt cargo-prebuilt```
- Cargo binstall: ```cargo binstall cargo-prebuilt --no-confirm```
- Cargo quickinstall: ```cargo quickinstall cargo-prebuilt```
- Homebrew: ```brew install crow-rest/harmless/cargo-prebuilt```
- Install script (unix platforms): ```curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-prebuilt/cargo-prebuilt/main/scripts/install-cargo-prebuilt.sh | bash```
- For github actions you can use [cargo-prebuilt/cargo-prebuilt-action](https://github.com/cargo-prebuilt/cargo-prebuilt-action)

## Building

(Cargo prebuilt requires a tls feature)

#### vendored-openssl (default)
```cargo install cargo-prebuilt```

#### native tls (github releases default)
```cargo install cargo-prebuilt --no-default-features --features default-native```

#### rustls
```cargo install cargo-prebuilt --no-default-features --features default-rustls```

#### rustls with native certs
```cargo install cargo-prebuilt --no-default-features --features default-rustls,rustls-native-certs```

### limit security used
(Cargo prebuilt is tested with default features and may break without the ```security``` feature)

Remove ```security``` feature included by default, then add the features you want below:
- ```sha2```: Sha2 hashing
- ```sha3```: Sha3 hashing
- ```sig```: Minisign signatures

#### limit indexes used
(Cargo prebuilt is tested with default features and may break without the ```indexes``` feature)

Remove ```indexes``` feature included by default, then add the features you want below:
- [github-public](#github-public)
- [github-private](#github-private) (Not supported yet)
- [gitlab-public](#gitlab-public) (Not supported yet)
- [gitlab-private](#gitlab-private) (Not supported yet)
- [forgejo-public](#forgejo-public) (Not supported yet)
- [forgejo-private](#forgejo-private) (Not supported yet)
- [gitea-public](#gitea-public) (Not supported yet)
- [gitea-private](#gitea-private) (Not supported yet)
- [custom-http-public](#custom-http-private) (Not supported yet)
- [custom-http-private](#custom-http-private) (Not supported yet)

#### limit color
(Cargo prebuilt is tested with default features and may break without the ```color``` feature)

- Remove the ```color``` feature (enabled by default)
- Or use ```--no-color```, ```NO_COLOR=true``` env var, or ```color = false``` in the config file.

### use mimalloc
```cargo install cargo-prebuilt --features mimalloc```

## Events

To output events use ```--out```.

See [Events](docs/EVENTS.md).

## Reports

Reports are generated when a crate is built in the index.

They are stored under ```$DATA/cargo-prebuilt/reports/$CRATE/$VERSION``` by default.

See [Data Dirs](https://docs.rs/directories/5.0.1/directories/struct.ProjectDirs.html#method.data_dir).

Use ```--report-path``` to change where they are stored.

Report types (--reports):
- license: Download license and put it under the prebuilt folder in the installation dir. (Default on)
- deps: Download deps tree and put it under the prebuilt folder in the installation dir.
- audit: Download audit and put it under the prebuilt folder in the installation dir.

## Using a custom index

#### GitHub public

[Template](https://github.com/cargo-prebuilt/gh-pub-index)

Your url should be formatted like ```github.com/cargo-prebuilt/index```. cargo-prebuilt requires https.

- ```export PREBUILT_INDEX=gh-pub:$URL```
- ```cargo prebuilt --index=gh-pub:$URL CRATES```
- [config.toml](docs/CONFIG.md) 
    ```toml
    [key.index]
    index = "gh-pub:$URL"
    ```

#### GitHub private

Under development
