use std::{collections::HashMap, path::PathBuf};

use indexmap::IndexSet;
use serde::{Deserialize, Serialize};

use super::HashType;

pub type SigKeys = HashMap<String, Vec<String>>;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ReportType {
    #[serde(rename = "license")]
    LicenseDL,
    #[serde(rename = "deps")]
    DepsDL,
    #[serde(rename = "audit")]
    AuditDL,
}
impl From<ReportType> for &str {
    fn from(value: ReportType) -> Self {
        match value {
            ReportType::LicenseDL => "license",
            ReportType::DepsDL => "deps",
            ReportType::AuditDL => "audit",
        }
    }
}
impl From<&ReportType> for &str {
    fn from(value: &ReportType) -> Self {
        Into::<&str>::into(*value)
    }
}
impl TryFrom<&str> for ReportType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "license" => Ok(ReportType::LicenseDL),
            "deps" => Ok(ReportType::DepsDL),
            "audit" => Ok(ReportType::AuditDL),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFileV1 {
    pub prebuilt: Option<ConfigFilePrebuiltV1>,
    pub key: Option<HashMap<String, ConfigFileKeysV1>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFilePrebuiltV1 {
    pub target: Option<String>,
    pub index: Option<String>,
    pub auth: Option<String>, //TODO: Should auth be base64 encoded in config file?
    pub path: Option<PathBuf>,
    pub report_path: Option<PathBuf>,
    pub no_create_path: Option<bool>,
    pub reports: Option<IndexSet<ReportType>>,
    pub color: Option<bool>,
    pub no_color: Option<bool>,
    pub hashes: Option<IndexSet<HashType>>,
    pub no_verify: Option<bool>,
    pub safe: Option<bool>,
    pub out: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFileKeysV1 {
    pub index: String,
    pub pub_key: String,
}

#[cfg(test)]
mod test {
    use super::ConfigFileV1;

    #[test]
    fn test_deser1() {
        let toml = include_str!("../../test/config_1.toml");
        let _: ConfigFileV1 = toml::from_str(toml).unwrap();
    }

    #[test]
    fn test_deser2() {
        let toml = include_str!("../../test/config_2.toml");
        let _: ConfigFileV1 = toml::from_str(toml).unwrap();
    }

    #[test]
    fn test_deser3() {
        let toml = "";
        let _: ConfigFileV1 = toml::from_str(toml).unwrap();
    }
}
