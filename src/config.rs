use crate::{TARGET, data::ConfigFileV1};
use bpaf::*;
use home::home_dir;
use std::{path::PathBuf, fs::File, io::Read};

pub static REPORT_FLAGS: [&str; 6] = [
    "license-out",
    "license-dl",
    "deps-out",
    "deps-dl",
    "audit-out",
    "audit-dl",
];

#[derive(Clone, Debug)]
pub struct Config {
    pub target: String,
    pub index: String,
    pub auth: Option<String>,
    pub path: PathBuf,
    pub report_path: PathBuf,
    pub ci: bool,
    pub no_create_path: bool,
    pub reports: String,
    pub color: bool,
    pub pkgs: String,
}

#[derive(Clone, Debug)]
struct Arguments {
    target: Option<String>,
    index: Option<String>,
    auth: Option<String>,
    path: Option<PathBuf>,
    report_path: Option<PathBuf>,
    ci: bool,
    no_create_path: Option<bool>,
    reports: Option<String>,
    color: Option<bool>,
    no_color: Option<bool>,
    pkgs: String,
}

fn parse_args() -> Arguments {
    let pkgs = positional::<String>("PKGS").help("A CSV list of packages with optional @VERSION");

    let target = long("target")
        .env("PREBUILT_TARGET")
        .help("Target of the binary to download. (Defaults to target of cargo-prebuilt)")
        .argument::<String>("TARGET")
        .optional();

    let index = long("index")
        .env("PREBUILT_INDEX")
        .help("Index to use. (Default: gh-pub:github.com/cargo-prebuilt/index)")
        .argument::<String>("INDEX")
        .optional();

    let auth = long("auth")
        .env("PREBUILT_AUTH")
        .help("Auth token to use for private indexes.")
        .argument::<String>("TOKEN")
        .optional();

    let path = long("path")
        .env("PREBUILT_PATH")
        .help("Path to the folder where downloaded binaries will be installed. (Default: $CARGO_HOME)")
        .argument::<PathBuf>("PATH")
        .optional();

    let report_path = long("report-path")
        .env("PREBUILT_REPORT_PATH")
        .help("Path to the folder where the reports will be put. (Default: $HOME/.prebuilt)")
        .argument::<PathBuf>("REPORT_PATH")
        .optional();

    let ci = long("ci")
        .env("PREBUILT_CI")
        .help("Do not download reports, create a .prebuilt directory, and check for a config file.")
        .switch();

    let no_create_path = long("no-create-path")
        .help("Do not create the report and/or bin folder if it is missing.")
        .switch()
        .optional();

    let reports = long("reports")
        .env("PREBUILT_REPORTS")
        .help("A CSV list of reports types. (license-out, license-dl, deps-out, deps-dl, audit-out, audit-dl)")
        .argument::<String>("REPORTS")
        .optional();

    let color = long("color")
        .help("Force color to be turned on.")
        .switch()
        .optional();

    let no_color = long("no-color")
        .help("Force color to be turned off.")
        .switch()
        .optional();

    let parser = construct!(Arguments {
        target,
        index,
        auth,
        path,
        report_path,
        ci,
        no_create_path,
        reports,
        color,
        no_color,
        pkgs,
    });
    
    cargo_helper("prebuilt", parser)
        .to_options()
        .version(env!("CARGO_PKG_VERSION"))
        .run()
}

fn fill_from_file(args: &mut Arguments) -> () {
    match home_dir() {
        Some(mut conf) => {
            conf.push(".config/cargo-prebuilt/config.toml");
            if conf.exists() {
                let mut file = File::open(conf).expect("Could not open config file.");
                let mut str = String::new();
                file.read_to_string(&mut str).expect("Could not read config file.");

                let config: Result<ConfigFileV1, toml::de::Error> = toml::from_str(&str);
                if let Ok(config) = config {
                    // TODO
                    todo!();
                }
            }
        },
        None => eprintln!("Could not find home directory! Config file will be ignored."),
    }
}

fn convert(args: &Arguments) -> Config {
    todo!()
}

pub fn get() -> Config {
    // arguments and env vars
    let mut args = parse_args();
    
    // config file
    fill_from_file(&mut args);
    
    // defaults
    
    convert(&args)
}
