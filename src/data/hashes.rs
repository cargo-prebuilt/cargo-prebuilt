use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub type Hashes = HashMap<HashType, String>;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum HashType {
    #[serde(rename = "sha256")]
    Sha256,
    #[serde(rename = "sha512")]
    Sha512,
    #[serde(rename = "sha3_256")]
    Sha3_256,
    #[serde(rename = "sha3_512")]
    Sha3_512,
}
impl From<HashType> for &str {
    fn from(value: HashType) -> Self {
        match value {
            HashType::Sha256 => "sha256",
            HashType::Sha512 => "sha512",
            HashType::Sha3_256 => "sha3_256",
            HashType::Sha3_512 => "sha3_512",
        }
    }
}
impl From<&HashType> for &str {
    fn from(value: &HashType) -> Self {
        Into::<&str>::into(*value)
    }
}
impl TryFrom<&str> for HashType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "sha256" => Ok(Self::Sha256),
            "sha512" => Ok(Self::Sha512),
            "sha3_256" => Ok(Self::Sha3_256),
            "sha3_512" => Ok(Self::Sha3_512),
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
    pub hashes: HashMap<String, HashesFileBlobV1>, // File hashes
}
impl From<HashesFile> for HashesFileV1 {
    fn from(value: HashesFile) -> Self {
        match value {
            HashesFile::V1(f) => f,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HashesFileBlobV1 {
    pub archive: Hashes,               // Archive Hashes
    pub bins: HashMap<String, Hashes>, // Binary Hashes
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
