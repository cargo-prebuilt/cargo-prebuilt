use std::collections::HashMap;

use crate::data::{Hashes, HashesFileBlobV1};
use serde::Deserialize;

/// This is an intermediate format, only for use in this program.
#[derive(Debug)]
pub struct InfoFileImm {
    pub id: String,                            // Crate ID
    pub version: String,                       // Crate Version
    pub license: String,                       // SPDX License String
    pub git: String,                           // Url to Git
    pub description: String,                   // Crate Description
    pub bins: Vec<String>,                     // Crate Binaries
    pub info: HashMap<String, String>,         // Metadata
    pub archive: InfoFileArchiveV2,            // Archive Info
    pub files: InfoFileFilesV2,                // File Names
    pub archive_name: String,                  // Archive Name
    pub archive_hashes: Hashes,                // Archive Hashes
    pub bins_hashes: HashMap<String, Hashes>,  // Binaries Hashes
    pub polyfill: Option<InfoFileImmPolyFill>, // Backwards Compat
}
impl InfoFileImm {
    pub fn convert(info: InfoFile, target: &String) -> Self {
        match info {
            InfoFile::V1(info) => Self {
                id: info.id,
                version: info.version,
                license: info.license,
                git: info.git,
                description: info.description,
                bins: info.bins,
                info: info.info,
                archive: InfoFileArchiveV2 {
                    compression: info.archive.compression,
                    package: "tar".to_string(),
                },
                files: InfoFileFilesV2 {
                    license: info.files.license,
                    deps: info.files.deps,
                    audit: info.files.audit,
                },
                archive_name: format!("{target}.{}", info.archive.ext),
                archive_hashes: HashMap::new(),
                bins_hashes: HashMap::new(),
                polyfill: Some(InfoFileImmPolyFill {
                    hash_file: info.files.hash,
                    hash_file_sig: info.files.sig_hash,
                }),
            },
            InfoFile::V2(info) => {
                let hashes = info
                    .hashes
                    .get(target)
                    .cloned()
                    .unwrap_or_else(|| HashesFileBlobV1 {
                        archive: HashMap::default(),
                        bins: HashMap::default(),
                    });

                Self {
                    id: info.id,
                    version: info.version,
                    license: info.license,
                    git: info.git,
                    description: info.description,
                    bins: info.bins,
                    info: info.info,
                    archive: info.archive,
                    files: info.files,
                    archive_name: info // TODO: Fail on target not found, here?
                        .targets
                        .get(target)
                        .cloned()
                        .unwrap_or_else(|| format!("{target}.tar.gz")),
                    archive_hashes: hashes.archive,
                    bins_hashes: hashes.bins,
                    polyfill: None,
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct InfoFileImmPolyFill {
    pub hash_file: String,
    pub hash_file_sig: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "info_version")]
pub enum InfoFile {
    #[serde(rename = "1")]
    V1(InfoFileV1),
    #[serde(rename = "2")]
    V2(InfoFileV2),
}

//region Info File V2
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfoFileV2 {
    pub id: String,                                // Crate ID
    pub version: String,                           // Crate Version
    pub license: String,                           // SPDX License String
    pub git: String,                               // Url to Git
    pub description: String,                       // Crate Description
    pub bins: Vec<String>,                         // Crate Binaries
    pub info: HashMap<String, String>,             // Metadata
    pub archive: InfoFileArchiveV2,                // Archive Info
    pub files: InfoFileFilesV2,                    // File Names
    pub targets: HashMap<String, String>,          // Targets Built For and File Names
    pub hashes: HashMap<String, HashesFileBlobV1>, // Hashes
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfoFileArchiveV2 {
    pub compression: String, // Archive Compression Type
    pub package: String,     // Archive Packing Type
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfoFileFilesV2 {
    pub license: String, // License File
    pub deps: String,    // Deps File
    pub audit: String,   // Audit File
}
//endregion

//region Info File V1
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfoFileV1 {
    pub id: String,                    // Crate ID
    pub version: String,               // Crate Version
    pub license: String,               // SPDX License String
    pub git: String,                   // Url to Git
    pub description: String,           // Crate Description
    pub bins: Vec<String>,             // Crate Binaries
    pub info: HashMap<String, String>, // Metadata
    pub archive: InfoFileArchiveV1,    // Archive Info
    pub files: InfoFileFilesV1,        // File Names
    pub targets: Vec<String>,          // Targets Built For
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfoFileArchiveV1 {
    pub compression: String, // Archive Compression Type
    pub ext: String,         // Archive Extension
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfoFileFilesV1 {
    pub hash: String,             // Hashes File
    pub license: String,          // License File
    pub deps: String,             // Deps File
    pub audit: String,            // Audit File
    pub sig_info: Option<String>, // Sig File For Info.json
    pub sig_hash: Option<String>, // Sig File For Hashes.json
}
//endregion

#[cfg(test)]
mod test {
    use super::InfoFile;

    #[test]
    fn test_deser1() {
        let json = include_str!("../../test/info_1.json");
        let _: InfoFile = serde_json::from_str(json).unwrap();
    }

    #[test]
    fn test_deser2() {
        let json = include_str!("../../test/info_2.json");
        let _: InfoFile = serde_json::from_str(json).unwrap();
    }
}
