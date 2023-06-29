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
    pub hash: String,    // Hashes File
    pub license: String, // License File
    pub deps: String,    // Deps File
    pub audit: String,   // Audit File
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
