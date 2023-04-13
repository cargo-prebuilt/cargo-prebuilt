use home::home_dir;
use std::{fs::File, io::Read};

pub fn get_index() -> Option<String> {
    let mut conf = home_dir()?;
    conf.push(".config/cargo-prebuilt/config.toml");

    if conf.exists() {
        let mut file = File::open(conf).expect("Could not open config file.");
        let mut str = String::new();
        file.read_to_string(&mut str)
            .expect("Could not read config file.");

        let t = nanoserde::TomlParser::parse(str.as_str()).expect("Could not parse config file.");
        if t.contains_key("prebuilt.index") {
            return Some(t.get("prebuilt.index")?.str().to_string());
        }
    }

    None
}
