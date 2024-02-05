# Changelog

## Upcoming

- Fix reports directory being created when --ci is used.

## [0.6.6](https://github.com/cargo-prebuilt/cargo-prebuilt/releases/tag/v0.6.6)

- Deprecate --gen-config.
- Update help messages to also point to docs and be more helpful.
- Fix the app qualifier to be `tech.harmless.cargo-prebuilt` instead of `tech.harmless.cargo-prebuilt.cargo-prebuilt`.
- Add report events.
- Add wrote report event.

## [0.6.5](https://github.com/cargo-prebuilt/cargo-prebuilt/releases/tag/v0.6.5)

- Drop sun solaris and windows gnu builds.
- Update to owo-colors 4.
- Update home.
- Bump MSRV from Rust 1.67+ to Rust 1.70+.

## [0.6.4](https://github.com/cargo-prebuilt/cargo-prebuilt/releases/tag/v0.6.4)

- Internally removed a lot of panics for better reliability.
- CLI color change.
- Panic on empty public keys when --no-verify is not used.
- Added GitHub Private index support. (beta)

## [0.6.3](https://github.com/cargo-prebuilt/cargo-prebuilt/releases/tag/v0.6.3)

- Changed config file to include auth on a per repo basis.
- Changed config file to better support multiple indexes.
- Changed how --gen-config works.
- Changed some of the cli colors.
- Bump MSRV from Rust 1.66+ to Rust 1.67+.

## [0.6.2](https://github.com/cargo-prebuilt/cargo-prebuilt/releases/tag/v0.6.2)

- Use vendored openssl 3 instead of openssl 1.1.1.
- Bump deps.
- Fix a warning message improperly appearing when there is a config.
- Add --require-config (-r) flag.
- Bump MSRV from Rust 1.63+ to Rust 1.66+.

## [0.6.1](https://github.com/cargo-prebuilt/cargo-prebuilt/releases/tag/v0.6.1)

- Bump deps.

## [0.6.0](https://github.com/cargo-prebuilt/cargo-prebuilt/releases/tag/v0.6.0)

- Changed how features work.
- Errors now use eprintln!().
- Changed error exit codes to be positive.
- Removed nanoserde in favor of serde.
- Bump deps.
- Bump MSRV from Rust 1.63+ to Rust 1.65+.
- version flag (--version) now uses bpaf's built in method.
- .prebuilt folder is now by default put in the home directory, but can be overridden.
- config.toml is now more useful. (See [CONFIG](CONFIG.md))
- Better config handling.
- Added support for sha3_256, sha3_512, and sha512.
- GitHub release builds now use native-tls instead of vendored openssl.
- Some 32-bit platforms were dropped from support. (#46)
- Added minisign key verifying for info.json and hashes.json.
- Verify by default and allow opt out.
- Uses abort for panics.
- File install locations are outputted into stdout. (#45)
- Switch to gen2 index. (#43)
- Reports can no longer be printed to stdout.
- Permissions are now explicitly set to 755 on unix platforms.
- To opt in or out of colors use the env vars FORCE_COLOR/NO_COLOR.
- Releases are signed using minisign.
- Color feature is now linked to owo-colors and supports-color.
- Remove a need for atty dependency.
- Add -s/--safe flag to prevent overwriting of binaries.
- Do not allow path separators in tar archives.
- Check binaries extracted from tar archive to make sure they are the correct ones.
- Allow adding to the config using the cmd line using --gen-config. (#76) (#77)
- Add --out flag that prints out json events. (#75)
- Follow directory standards. (#79)
- Add --index-key which will select an index based on its entry in the config file. (#78)
- Add --config to allow selection of a config file anywhere. (#80)
- Use panics instead of using std::process::exit. (#47)
- Add way to get latest versions of crates. (#84)

## [0.5.3](https://github.com/cargo-prebuilt/cargo-prebuilt/releases/tag/v0.5.3)

- Boost MSRV to Rust 1.63+ from Rust 1.60+.
- Allow for rustls to be used instead of just openssl.
- Errors on compile when there are no index features selected.

## [0.5.2](https://github.com/cargo-prebuilt/cargo-prebuilt/releases/tag/v0.5.2)

- Support optional config file under $HOME/.config/cargo-prebuilt/config.toml.

## [0.5.1](https://github.com/cargo-prebuilt/cargo-prebuilt/releases/tag/v0.5.1)

- Same as 0.5.0.

## [0.5.0](https://github.com/cargo-prebuilt/cargo-prebuilt/releases/tag/v0.5.0)

- Switch to bpaf for argument parsing. (#20)
- Add env vars for some options. (#20)
- Add owo-colors. (#20)
- Support for custom indexes. [Github public] (#24)
- Removed which/where detection.
- Check if path exists before downloading.

## [0.4.2](https://github.com/cargo-prebuilt/cargo-prebuilt/releases/tag/v0.4.2)

- Initial Changelog.
