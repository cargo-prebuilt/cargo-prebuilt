# Index Standards

Pulled from [API.md](https://github.com/cargo-prebuilt/index/blob/main/API.md).

## V1

### info.json v1

- Only gzip is supported for compression.
- Only tar is supported for packaging.
- Sig file must be under `$INFO_JSON_URL.minisig` for cargo-prebuilt versions
  \> 0.6.\*

```json
{
  "info_version": "1",
  "id": "CRATES.IO ID",
  "version": "VERSION",
  "license": "SPDX LICENSE",
  "git": "GIT URL",
  "description": "CRATES.IO DESCRIPTION",
  "bins": ["BINARY", "BINARY?"],
  "info": {
    "META": "DATA"
  },
  "archive": {
    "compression": "gz",
    "ext": "tar.gz"
  },
  "files": {
    "hash": "hashes.json",
    "license": "license.report",
    "deps": "deps.report",
    "audit": "audit.report",
    "sig_info": "OPTIONAL:info.json.minisig",
    "sig_hashes": "OPTIONAL:hashes.json.minisig"
  },
  "targets": ["TARGET", "TARGET?"]
}
```

### hashes.json v1

```json
{
  "hashes_version": "1",
  "hashes": {
    "TARGET": {
      "archive": {
        "HASH_TYPE": "HASH",
        "HASH_TYPE?": "HASH?"
      },
      "bins": {
        "BINARY": {
          "HASH_TYPE": "HASH",
          "HASH_TYPE?": "HASH?"
        },
        "BINARY?": {
          "HASH_TYPE": "HASH",
          "HASH_TYPE?": "HASH?"
        }
      }
    },
    "TARGET?": {
      "archive": {
        "HASH_TYPE": "HASH",
        "HASH_TYPE?": "HASH?"
      },
      "bins": {
        "BINARY": {
          "HASH_TYPE": "HASH",
          "HASH_TYPE?": "HASH?"
        },
        "BINARY?": {
          "HASH_TYPE": "HASH",
          "HASH_TYPE?": "HASH?"
        }
      }
    }
  }
}
```

## V2

### info.json v2

- Only gzip is supported for compression.
- Only tar is supported for packaging.
- Sig file must be under `$INFO_JSON_URL.minisig`

```json
{
  "info_version": "2",
  "id": "CRATES.IO ID",
  "version": "VERSION",
  "license": "SPDX LICENSE",
  "git": "GIT URL",
  "description": "CRATES.IO DESCRIPTION",
  "bins": ["BINARY", "BINARY?"],
  "info": {
    "META": "DATA"
  },
  "archive": {
    "compression": "gz",
    "package": "tar"
  },
  "files": {
    "license": "license.report",
    "deps": "deps.report",
    "audit": "audit.report"
  },
  "targets": {
    "TARGET": "FILE NAME"
  },
  "hashes": {
    "TARGET": {
      "archive": {
        "HASH_TYPE": "HASH",
        "HASH_TYPE?": "HASH?"
      },
      "bins": {
        "BINARY": {
          "HASH_TYPE": "HASH",
          "HASH_TYPE?": "HASH?"
        },
        "BINARY?": {
          "HASH_TYPE": "HASH",
          "HASH_TYPE?": "HASH?"
        }
      }
    },
    "TARGET?": {
      "archive": {
        "HASH_TYPE": "HASH",
        "HASH_TYPE?": "HASH?"
      },
      "bins": {
        "BINARY": {
          "HASH_TYPE": "HASH",
          "HASH_TYPE?": "HASH?"
        },
        "BINARY?": {
          "HASH_TYPE": "HASH",
          "HASH_TYPE?": "HASH?"
        }
      }
    }
  }
}
```
