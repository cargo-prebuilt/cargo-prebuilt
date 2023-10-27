use std::{collections::HashMap, path::PathBuf};

use indexmap::IndexSet;
use serde::{Deserialize, Serialize};

use super::HashType;

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
pub struct ConfigFile {
    pub prebuilt: Option<ConfigFilePrebuilt>,
    pub index: Option<HashMap<String, ConfigFileIndexes>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFilePrebuilt {
    pub target: Option<String>,
    pub index_key: Option<String>,
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
pub struct ConfigFileIndexes {
    pub index: String,
    pub pub_key: Option<Vec<String>>,
    pub auth: Option<String>, // TODO: Should be stored in base64? Maybe encrypt?
}

#[cfg(test)]
mod test {
    use super::ConfigFile;

    #[test]
    fn test_deser1() {
        let toml = include_str!("../../test/config_1.toml");
        let _: ConfigFile = toml::from_str(toml).unwrap();
    }

    #[test]
    fn test_deser2() {
        let toml = include_str!("../../test/config_2.toml");
        let _: ConfigFile = toml::from_str(toml).unwrap();
    }

    #[test]
    fn test_deser3() {
        let toml = "";
        let _: ConfigFile = toml::from_str(toml).unwrap();
    }
}
