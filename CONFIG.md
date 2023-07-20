# Config Info

Config info is prioritized in the order of arguments -> environmental variables -> config file -> defaults.

## Args

Use ```cargo prebuilt --help```.

## Environmental Vars

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
color = true|false          # Should CLI be on
no_color = true|false       # Should CLI colors be off
no_verify = true|false      # Do not verify signatures (See [key.$ANYTHING])
safe = true|false           # Prevent the overwriting of binaires (Except when --ci is used)

[key.$ANYTHING]             # Add a public verifying key for an index
index = "$INDEX"            # Index to add key for
pub_key = "$PUBLIC_KEY"     # Public minisign verifying key for index (No comments)
```

## Ref

- ```$TARGET``` is a rustc target string. EX: ```aarch64-apple-darwin```
- ```$INDEX``` is a custom index string. EX: ```gh-pub:github.com/cargo-prebuilt/index```
- ```$TOKEN``` is a auth token for the index.
- ```$PATH``` is a absolute path. EX: ```/User/devops/.cargo/bin```
- ```$REPORT_TYPE``` is a type of report. ```license, deps, audit```
- ```$PUBLIC_KEY``` is a public minisign key. (See keys/cargo-prebuilt-index.pub)
