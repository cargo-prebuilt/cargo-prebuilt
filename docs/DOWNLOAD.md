# Downloads

## Download Methods

- You can download the latest prebuilt binaries of cargo-prebuilt
  [here](https://github.com/cargo-prebuilt/cargo-prebuilt/releases/latest).
- Cargo install: `cargo install cargo-prebuilt` or
  `cargo install cargo-prebuilt --profile=quick-build`
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

## Verify

If you are downloading cargo-prebuilt from the release page then there are
multiple ways to verify it.

### Minisign

Download the associated minisig file with the archive you picked.

Get the [public key](https://github.com/cargo-prebuilt/cargo-prebuilt/blob/main/keys/cargo-prebuilt.pub.base64).

Download [minisign](https://jedisct1.github.io/minisign/) or
[rsign2](https://github.com/jedisct1/rsign2) and follow their instructions.

### GitHub Attestation

Download the [GitHub CLI](https://cli.github.com/) and run
`gh attestation verify $ARCHIVE --repo cargo-prebuilt/cargo-prebuilt`.

### Hash File

Download the associated hashes.sha256 file and run
`sha256sum -c --ignore-missing hashes.sha256`.
