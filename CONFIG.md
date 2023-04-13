# Config Info

Config info is process in the order of arguments -> environmental variables -> config file -> defaults.

## File

The config file is under ```$HOME/.config/cargo-prebuilt/config.toml```.

```toml
[prebuilt]
index = "$INDEX"
```

## Ref

- ```$INDEX``` is a custom index string. EX: ```gh-pub:github.com/cargo-prebuilt/index```
