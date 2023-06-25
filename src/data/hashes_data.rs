use std::collections::HashMap;

use serde::Deserialize;

use super::HashType;

#[derive(Debug, Deserialize)]
#[serde(tag = "hashes_version")]
pub enum HashesFile {
    #[serde(rename = "1")]
    V1(HashesFileV1),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HashesFileV1 {
    hashes: HashMap<String, HashesFileBlobV1>, // File hashes
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HashesFileBlobV1 {
    archive: HashMap<String, HashMap<HashType, String>>,
    bin: HashMap<String, HashMap<HashType, String>>,
}
