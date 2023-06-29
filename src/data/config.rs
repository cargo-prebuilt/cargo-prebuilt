use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFileV1 {
    pub prebuilt: ConfigFilePrebuiltV1,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFilePrebuiltV1 {
    pub index: Option<String>,
}

#[cfg(test)]
mod test {
    use super::ConfigFileV1;

    #[test]
    fn test_deser1() {
        let toml = include_str!("../../test/config_1.toml");
        let _: ConfigFileV1 = toml::from_str(toml).unwrap();
    }
}
