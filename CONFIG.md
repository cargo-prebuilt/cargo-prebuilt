# Config Info

Config info is prioritized in the order of arguments -> environmental variables -> config file -> defaults.

## Args

Use ```cargo prebuilt --help```.

## File

The config file should be under ```$HOME/.config/cargo-prebuilt/config.toml```.

The config file is ignored when using the --ci flag.

```toml
[prebuilt]
target = "$TARGET"          # Target to download for
index = "$INDEX"            # Index to use
auth = "$TOKEN"             # Index auth token
path = "$PATH"              # Absolute path to where the binaries will be installed
report_path = "$PATH"       # Absolute path to where the reports will be put
no_create_path = true|false # Do not create paths that do not exist
reports = ["$REPORT_TYPE"]  # Reports to download
hashes = ["$HASH_TYPE"]     # Hashes to use for verifying files downloaded
color = true|false          # Should CLI colors be on or not
force_sig = true|false      # Force verifying signatures to be used (See [key.$ANYTHING])

[key.$ANYTHING]             # Add a public verifying key for an index
index = "$INDEX"            # Index to add key for
pub_key = "$PUBLIC_GPG_KEY" # Public verifying key for index
```

## Ref

- ```$TARGET``` is a rustc target string. EX: ```aarch64-apple-darwin```
- ```$INDEX``` is a custom index string. EX: ```gh-pub:github.com/cargo-prebuilt/index```
- ```$TOKEN``` is a auth token for the index.
- ```$PATH``` is a absolute path. EX: ```/User/devops/.cargo/bin```
- ```$REPORT_TYPE``` is a type of report. ```license-out, license-dl, deps-out, deps-dl, audit-out, audit-dl```
- ```$HASH_TYPE``` is a type of hash. ```sha256, sha512, sha3_256, sha3_512```
