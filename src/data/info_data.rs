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
    description: String,           // Crate Description
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
    hashes: String,  // Hashes file
    license: String, // License File
    deps: String,    // Deps File
    audit: String,   // Audit File
}
