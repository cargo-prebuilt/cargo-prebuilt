use std::collections::HashMap;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash)]
pub enum HashType {
    #[cfg(feature = "sha2")]
    #[serde(rename = "sha256")]
    Sha256,
    #[cfg(feature = "sha2")]
    #[serde(rename = "sha512")]
    Sha512,
    #[cfg(feature = "sha3")]
    #[serde(rename = "sha3_256")]
    Sha3_256,
    #[cfg(feature = "sha3")]
    #[serde(rename = "sha3_512")]
    Sha3_512,
}
impl From<HashType> for &str {
    fn from(value: HashType) -> Self {
        match value {
            #[cfg(feature = "sha2")]
            HashType::Sha256 => "sha256",
            #[cfg(feature = "sha2")]
            HashType::Sha512 => "sha512",
            #[cfg(feature = "sha3")]
            HashType::Sha3_256 => "sha3_256",
            #[cfg(feature = "sha3")]
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
            #[cfg(feature = "sha2")]
            "sha256" => Ok(HashType::Sha256),
            #[cfg(feature = "sha2")]
            "sha512" => Ok(HashType::Sha512),
            #[cfg(feature = "sha3")]
            "sha3_256" => Ok(HashType::Sha3_256),
            #[cfg(feature = "sha3")]
            "sha3_512" => Ok(HashType::Sha3_512),
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HashesFileBlobV1 {
    pub archive: HashMap<String, String>, // Archive Hashes
    pub bins: HashMap<String, HashMap<HashType, String>>, // Binary Hashes
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
