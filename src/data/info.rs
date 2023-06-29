use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "info_version")]
pub enum InfoFile {
    #[serde(rename = "1")]
    V1(InfoFileV1),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfoFileV1 {
    id: String,                    // Crate ID
    version: String,               // Crate Version
    license: String,               // SPDX License String
    git: String,                   // Url to Git
    description: String,           // Crate Description
    bins: Vec<String>,             // Crate Binaries
    info: HashMap<String, String>, // Metadata
    archive: InfoFileArchiveV1,    // Archive Info
    files: InfoFileFilesV1,        // File Names
    targets: Vec<String>,          // Targets Built For
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfoFileArchiveV1 {
    compression: String, // Archive compression type
    ext: String,         // Archive extension
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfoFileFilesV1 {
    hash: String,    // Hashes file
    license: String, // License File
    deps: String,    // Deps File
    audit: String,   // Audit File
}

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
