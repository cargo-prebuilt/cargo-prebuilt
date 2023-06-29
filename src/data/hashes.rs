use std::collections::HashMap;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash)]
pub enum HashType {
    #[serde(rename = "sha256")]
    Sha256,
    #[serde(rename = "sha512")]
    Sha512,
}
impl From<HashType> for &str {
    fn from(value: HashType) -> Self {
        match value {
            HashType::Sha256 => "sha256",
            HashType::Sha512 => "sha512",
        }
    }
}
impl TryFrom<&str> for HashType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "sha256" => Ok(HashType::Sha256),
            "sha512" => Ok(HashType::Sha512),
            _ => Err(()),
        }
    }
}

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
    archive: HashMap<String, String>,
    bins: HashMap<String, HashMap<HashType, String>>,
}

#[cfg(test)]
mod test {
    use super::HashesFile;

    #[test]
    fn test_deser1() {
        let json = include_str!("../../test/hashes_1.json");
        let _: HashesFile = serde_json::from_str(json).unwrap();
    }

    #[test]
    fn test_deser2() {
        let json = include_str!("../../test/hashes_2.json");
        let _: HashesFile = serde_json::from_str(json).unwrap();
    }
}
