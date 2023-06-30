use std::path::PathBuf;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash)]
pub enum ReportType {
    #[serde(rename = "license-out")]
    LicenseOut,
    #[serde(rename = "license-dl")]
    LicenseDL,
    #[serde(rename = "deps-out")]
    DepsOut,
    #[serde(rename = "deps-dl")]
    DepsDL,
    #[serde(rename = "audit-out")]
    AuditOut,
    #[serde(rename = "audit-dl")]
    AuditDL,
}
impl From<ReportType> for &str {
    fn from(value: ReportType) -> Self {
        match value {
            ReportType::LicenseOut => "license-out",
            ReportType::LicenseDL => "license-dl",
            ReportType::DepsOut => "deps-out",
            ReportType::DepsDL => "deps-dl",
            ReportType::AuditOut => "audit-out",
            ReportType::AuditDL => "audit-dl",
        }
    }
}
impl TryFrom<&str> for ReportType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "license-out" => Ok(ReportType::LicenseOut),
            "license-dl" => Ok(ReportType::LicenseDL),
            "deps-out" => Ok(ReportType::DepsOut),
            "deps-dl" => Ok(ReportType::DepsDL),
            "audit-out" => Ok(ReportType::AuditOut),
            "audit-dl" => Ok(ReportType::AuditDL),
            _ => Err(())
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFileV1 {
    pub prebuilt: Option<ConfigFilePrebuiltV1>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFilePrebuiltV1 {
    pub target: Option<String>,
    pub index: Option<String>,
    pub index_token: Option<String>,
    pub path: Option<PathBuf>,
    pub report_path: Option<PathBuf>,
    pub no_create_path: Option<bool>,
    pub reports: Option<Vec<ReportType>>,
    pub color: Option<bool>,
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
