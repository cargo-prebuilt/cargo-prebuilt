use home::home_dir;
use std::{fs::File, io::Read};

//use crate::{data::ConfigFileV1, args::Arguments};
//
//pub fn fill(args: &mut Arguments) {
////    args.index = conf_file::get_index();
//
//}

// TODO: Needs rework for additional config options.
//pub fn get_index() -> Option<String> {
//    let mut conf = home_dir()?;
//    conf.push(".config/cargo-prebuilt/config.toml");
//
//    if conf.exists() {
//        let mut file = File::open(conf).expect("Could not open config file.");
//        let mut str = String::new();
//        file.read_to_string(&mut str)
//            .expect("Could not read config file.");
//
//        let config: ConfigFileV1 = toml::from_str(&str).unwrap_or(None)?;
//        return config.prebuilt.index;
//    }
//
//    None
//}
