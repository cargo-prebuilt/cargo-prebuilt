# Config Info

Config info is prioritized in the order of:
1. Arguments
2. Environmental Vars
3. Config File
4. Defaults

## Args

Use ```cargo prebuilt --help```.

## Environmental Vars

Use ```cargo prebuilt --help```.

## File

> [!IMPORTANT]
> The config file is ignored when using the --ci flag.

> [!WARNING]
> Config files are not stable between any versions.

```toml
TODO
```

## Types

### Report Types
Ref: $REPORT_TYPE
- audit_dl : Download the audit file to the data reports directory
- audit_out
- deps_dl
- deps_out
- license_dl
- license_out

