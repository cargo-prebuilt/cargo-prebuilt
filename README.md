# cargo-prebuilt

[![Rust Build and Test](https://github.com/crow-rest/cargo-prebuilt/actions/workflows/build.yml/badge.svg?event=push)](https://github.com/crow-rest/cargo-prebuilt/actions/workflows/build.yml)
[![Rust Checks](https://github.com/crow-rest/cargo-prebuilt/actions/workflows/checks.yml/badge.svg?event=push)](https://github.com/crow-rest/cargo-prebuilt/actions/workflows/checks.yml)
[![Crates.io](https://img.shields.io/crates/v/cargo-prebuilt)](https://crates.io/crates/cargo-prebuilt)

Download prebuilt binaries of some crate.io crates.

See supported targets and a list of prebuilt crates [here](https://github.com/crow-rest/cargo-prebuilt-index).

*(Some targets may not be prebuilt for some crates)*

Request a crate to be added [here](https://github.com/crow-rest/cargo-prebuilt-index/issues/new?assignees=&labels=add-crate%2C+under-consideration&template=request-crate.md&title=).

## How to Use

To download a crate:
```cargo prebuilt CRATE_NAME```

To download multiple crates:
```cargo prebuilt CRATE_1,CRATE_2,CRATE_3,...```

To download a version of a crate:
```cargo prebuilt CRATE_NAME@VERSION```

To download multiple crates with versions:
```cargo prebuilt CRATE_1@V1,CRATE_2,CRATE_3@V3,...```

Report types:
- license-out: Print license to stdout.
- license-dl: Download license and put it under the prebuilt folder in the installation dir. (Default on)
- deps-out: Print deps tree to stdout.
- deps-dl: Download deps tree and put it under the prebuilt folder in the installation dir.
- audit-out: Print audit to stdout.
- audit-dl: Download audit and put it under the prebuilt folder in the installation dir.

Need help? Try:
```cargo prebuilt --help```

## Download

You can download the latest prebuilt binaries of cargo-prebuilt [here](https://github.com/crow-rest/cargo-prebuilt/releases/latest).

## Current Limitations

- Cargo detection has not been tested on windows.
- When using a crate version it must be the exact semver version.
- Does not use *cargo install* or *cargo-binstall* as a backup.
