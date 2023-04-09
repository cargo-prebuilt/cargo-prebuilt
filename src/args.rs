use crate::TARGET;
use bpaf::*;
use std::path::PathBuf;

pub static REPORT_FLAGS: [&str; 6] = [
    "license-out",
    "license-dl",
    "deps-out",
    "deps-dl",
    "audit-out",
    "audit-dl",
];

#[derive(Clone, Debug)]
pub struct Arguments {
    pub target: String,
    pub index: Option<String>,
    pub auth: Option<String>,
    pub path: Option<PathBuf>,
    pub no_bin: bool,
    pub ci: bool,
    pub no_create_home: bool,
    pub reports: String,
    pub version: bool,
    pub colors: bool,
    pub no_colors: bool,
    pub pkgs: String,
}

pub fn parse_args() -> Arguments {
    let pkgs = positional::<String>("PKGS").help("A CSV list of packages with optional @VERSION");

    let target = long("target")
        .env("PREBUILT_TARGET")
        .help("Target of the binary to download. (Defaults to target of cargo-prebuilt)")
        .argument::<String>("TARGET")
        .fallback(TARGET.to_string());

    let index = long("index")
        .env("PREBUILT_INDEX")
        .help("Index to use. (Default: github.com/cargo-prebuilt/index)")
        .argument::<String>("INDEX")
        .optional();

    let auth = long("auth")
        .env("PREBUILT_AUTH")
        .help("Auth token to use for private indexes")
        .argument::<String>("TOKEN")
        .optional();

    let path = long("path")
        .env("PREBUILT_HOME")
        .env("CARGO_HOME")
        .help("Path to the home folder where downloaded binaries will be installed")
        .argument::<PathBuf>("PATH")
        .optional();

    let no_bin = long("no-bin")
        .help("Do not add /bin to the end of the path")
        .switch();

    let ci = long("ci")
        .help("Do not download reports or create a .prebuilt directory")
        .switch();

    let no_create_home = long("no-create-home")
        .help("Do not create the home an/or bin folder if it is missing")
        .switch();

    let reports = long("reports")
        .env("PREBUILT_REPORTS")
        .help("A CSV list of reports types. (license-out, license-dl, deps-out, deps-dl, audit-out, audit-dl)")
        .argument::<String>("REPORTS")
        .fallback("license-dl".to_string());

    let version = short('v')
        .long("version")
        .help("Prints out program version")
        .switch();

    let colors = long("colors").help("Force colors to be turned on").switch();

    let no_colors = long("no-colors")
        .help("Force colors to be turned off")
        .switch();

    let parser = construct!(Arguments {
        target,
        index,
        auth,
        path,
        no_bin,
        ci,
        no_create_home,
        reports,
        version,
        colors,
        no_colors,
        pkgs,
    });

    cargo_helper("prebuilt", parser).to_options().run()
}
