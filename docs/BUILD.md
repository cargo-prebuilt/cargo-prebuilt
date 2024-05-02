# Building cargo-prebuilt

## Building

`cargo build` for a debug version.

`cargo build --release` for a release version.

`cargo build --profile=small` for a smaller release version.

`cargo build --profile=quick-build` for a faster building release version,
but a much bigger binary size with optimizations turned off.

## Build Time Env Vars

- `PREBUILT_BUILD_REPO_LINK`: Sets the repo link that appears under --version.
- `PREBUILT_BUILD_ISSUES_LINK`: Sets the issues link that appears under --version.
- `PREBUILT_BUILD_DOCS_LINK`: Sets the docs link that appears under --version and --docs.
- `PREBUILT_BUILD_DEFAULT_INDEX`: Sets the default index to pull from.
- `PREBUILT_BUILD_DEFAULT_INDEX_KEY`: Sets the public keys to use for the default index. (CSV)
- `PREBUILT_BUILD_DEFAULT_TARGET`: Sets the default target.
