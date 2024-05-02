use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use indexmap::IndexSet;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ReportType {
    #[serde(rename = "license")]
    LicenseDL,
    #[serde(rename = "deps")]
    DepsDL,
    #[serde(rename = "audit")]
    AuditDL,
    #[serde(rename = "info_json")]
    InfoJsonDL,
    #[serde(rename = "license_event")]
    LicenseEvent,
    #[serde(rename = "deps_event")]
    DepsEvent,
    #[serde(rename = "audit_event")]
    AuditEvent,
    #[serde(rename = "info_json_event")]
    InfoJsonEvent,
}
impl From<ReportType> for &str {
    fn from(value: ReportType) -> Self {
        match value {
            ReportType::LicenseDL => "license",
            ReportType::DepsDL => "deps",
            ReportType::AuditDL => "audit",
            ReportType::InfoJsonDL => "info_json",
            ReportType::LicenseEvent => "license_event",
            ReportType::DepsEvent => "deps_event",
            ReportType::AuditEvent => "audit_event",
            ReportType::InfoJsonEvent => "info_json_event",
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
            "license" => Ok(Self::LicenseDL),
            "deps" => Ok(Self::DepsDL),
            "audit" => Ok(Self::AuditDL),
            "info_json" => Ok(Self::InfoJsonDL),
            "license_event" => Ok(Self::LicenseEvent),
            "deps_event" => Ok(Self::DepsEvent),
            "audit_event" => Ok(Self::AuditEvent),
            "info_json_event" => Ok(Self::InfoJsonEvent),
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
    pub safe: Option<bool>,
    pub index_key: Option<String>,
    pub no_sig: Option<bool>,
    pub no_hash: Option<bool>,
    pub hash_bins: Option<bool>,
    pub path: Option<PathBuf>,
    pub report_path: Option<PathBuf>,
    pub no_create_path: Option<bool>,
    pub reports: Option<IndexSet<ReportType>>,
    pub out: Option<bool>,
    pub color: Option<bool>,
    pub no_color: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFileIndexes {
    pub index: String,
    pub pub_key: Option<HashSet<String>>,
    pub auth: Option<String>, // TODO: Should be stored in base64?
}

#[cfg(test)]
mod test {
    use super::ConfigFile;

    #[test]
    fn test_deser1() {
        let toml = include_str!("../../test/config_1.toml");
        let _: ConfigFile = basic_toml::from_str(toml).unwrap();
    }

    #[test]
    fn test_deser2() {
        let toml = include_str!("../../test/config_2.toml");
        let _: ConfigFile = basic_toml::from_str(toml).unwrap();
    }

    #[test]
    fn test_deser3() {
        let toml = "";
        let _: ConfigFile = basic_toml::from_str(toml).unwrap();
    }
}
