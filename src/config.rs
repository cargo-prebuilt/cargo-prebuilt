use crate::{
    color::{self, err_color_print, PossibleColor},
    data::{ConfigFile, ReportType},
    APPLICATION, DEFAULT_INDEX, ORG, QUALIFIER, TARGET,
};
use directories::ProjectDirs;
use home::cargo_home;
use indexmap::IndexSet;
use std::{collections::HashSet, fs::File, io::Read, path::PathBuf};

static CONFIG_FILE: &str = "config.toml";

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug)]
pub struct Config {
    pub target: String,
    pub index: String,
    pub auth: Option<String>,
    pub path: PathBuf,
    pub report_path: PathBuf,
    pub ci: bool,
    pub no_create_path: bool,
    pub reports: IndexSet<ReportType>,
    pub sigs: HashSet<String>,
    pub no_verify: bool,
    pub safe: bool,
    pub out: bool,
    pub get_latest: bool,
    pub pkgs: IndexSet<String>,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug)]
struct Arguments {
    target: Option<String>,
    index_key: Option<String>,
    index: Option<String>,
    auth: Option<String>,
    config: Option<PathBuf>,
    path: Option<PathBuf>,
    report_path: Option<PathBuf>,
    ci: bool,
    no_create_path: bool,
    reports: Option<IndexSet<ReportType>>,
    pub_key: HashSet<String>,
    no_verify: bool,
    safe: bool,
    out: bool,
    color: bool,
    no_color: bool,
    get_latest: bool,
    require_config: bool,
    pkgs: IndexSet<String>,
}

// TODO: Consider moving fallback/default values to here.
#[allow(clippy::too_many_lines)]
fn parse_args() -> Arguments {
    #[allow(clippy::wildcard_imports)]
    use bpaf::*;

    let pkgs = positional::<String>("PKGS")
        .help("A CSV list of packages with optional @VERSION")
        .parse(|s| {
            let mut v = IndexSet::new();
            for i in s.split(',') {
                v.insert(i.to_string());
            }
            Ok::<IndexSet<String>, String>(v)
        });

    let target = long("target")
        .env("PREBUILT_TARGET")
        .help(format!("Target of the binary to download. (Default: {TARGET})").as_str())
        .argument::<String>("TARGET")
        .optional();

    let index_key = long("index-key")
        .env("PREBUILT_INDEX_KEY")
        .help("Index to use, pulling from config file. Overrides --index.")
        .argument::<String>("INDEX_KEY")
        .optional();

    let index = long("index")
        .env("PREBUILT_INDEX")
        .help(format!("Index to use. (Default: {DEFAULT_INDEX})").as_str())
        .argument::<String>("INDEX")
        .optional();

    let auth = long("auth")
        .env("PREBUILT_AUTH")
        .help("Auth token to use for private indexes.")
        .argument::<String>("TOKEN")
        .optional();

    let config = long("config")
        .env("PREBUILT_CONFIG")
        .help(format!("Path to the config file (Default: See https://github.com/cargo-prebuilt/cargo-prebuilt/blob/v{}/docs/PATHS.md#config)", env!("CARGO_PKG_VERSION")).as_str())
        .argument::<PathBuf>("CONFIG_PATH")
        .optional();

    let path = long("path")
        .env("PREBUILT_PATH")
        .help("Path to the folder where downloaded binaries will be installed. (Default: $CARGO_HOME/bin)")
        .argument::<PathBuf>("PATH")
        .optional();

    let report_path = long("report-path")
        .env("PREBUILT_REPORT_PATH")
        .help(format!("Path to the folder where the reports will be put (Default: See https://github.com/cargo-prebuilt/cargo-prebuilt/blob/v{}/docs/PATHS.md#reports)", env!("CARGO_PKG_VERSION")).as_str())
        .argument::<PathBuf>("REPORT_PATH")
        .optional();

    let ci = long("ci")
        .env("PREBUILT_CI")
        .help("Do not download reports, check for a config file, and ignore safe mode.")
        .switch();

    let no_create_path = long("no-create-path")
        .env("PREBUILT_NO_CREATE_PATH")
        .help("Do not create the report and/or bin folder if it is missing.")
        .switch();

    let reports = long("reports")
        .env("PREBUILT_REPORTS")
        .help(format!("Reports to be downloaded in a CSV format (Default: license) (See: See https://github.com/cargo-prebuilt/cargo-prebuilt/blob/v{}/docs/REPORT_TYPES.md)", env!("CARGO_PKG_VERSION")).as_str())
        .argument::<String>("REPORTS")
        .parse(|s| {
            let mut v = IndexSet::new();
            if !s.eq("") {
                for i in s.split(',') {
                    match TryInto::<ReportType>::try_into(i) {
                        Ok(d) => {
                            let _ = v.insert(d);
                        }
                        Err(()) => return Err(format!("{i} is not a report type.")),
                    }
                }
            }
            Ok(v)
        })
        .optional();

    let pub_key = long("pub-key")
        .env("PREBUILT_PUB_KEY")
        .help("A public verifying key encoded as base64. Must be used with --index.")
        .argument::<String>("PUB_KEY")
        .map(|s| {
            s.split(',')
                .map(std::borrow::ToOwned::to_owned)
                .collect::<HashSet<_>>()
        })
        .fallback(HashSet::new());

    let no_verify = long("no-verify")
        .env("PREBUILT_NO_VERIFY")
        .help("Do not verify downloaded info.json's and hashes.json's.")
        .switch();

    let safe = short('s')
        .long("safe")
        .env("PREBUILT_SAFE")
        .help("Do not overwrite binaries that already exist.")
        .switch();

    let out = long("out")
        .env("PREBUILT_OUT")
        .help("Output events.")
        .switch();

    let color = long("color")
        .env("FORCE_COLOR")
        .help("Force color to be turned on.")
        .switch();

    let no_color = long("no-color")
        .env("NO_COLOR")
        .help("Force color to be turned off.")
        .switch();

    let get_latest = long("get-latest")
        .env("PREBUILT_GET_LATEST")
        .help("Get latest versions of crates in index and then exit.")
        .switch();

    let require_config = short('r')
        .long("require-config")
        .env("PREBUILT_REQUIRE_CONFIG")
        .help("Require a config file to be used. (--ci will override this)")
        .switch();

    // TODO: sig-with and verify-with

    let parser = construct!(Arguments {
        target,
        index_key,
        index,
        auth,
        config,
        path,
        report_path,
        ci,
        no_create_path,
        reports,
        pub_key,
        no_verify,
        safe,
        out,
        color,
        no_color,
        get_latest,
        require_config,
        pkgs,
    });

    cargo_helper("prebuilt", parser)
        .to_options()
        .version(env!("CARGO_PKG_VERSION"))
        .run()
}

#[allow(clippy::cognitive_complexity)]
fn fill_from_file(args: &mut Arguments) {
    let conf = if let Some(p) = args.config.clone() {
        p
    }
    else if args.config.is_none() {
        if let Some(project) = ProjectDirs::from(QUALIFIER, ORG, APPLICATION) {
            let mut conf = PathBuf::from(project.config_dir());
            conf.push(CONFIG_FILE);
            conf
        }
        else {
            eprintln!("Could not find default config directory! Config file will be ignored.");
            return;
        }
    }
    else {
        unreachable!()
    };

    if conf.exists() {
        let mut file = File::open(&conf).expect("Could not open config file.");
        let mut str = String::new();
        file.read_to_string(&mut str)
            .expect("Could not read config file.");

        let config: Result<ConfigFile, toml::de::Error> = toml::from_str(&str);
        match config {
            Ok(config) => {
                if let Some(prebuilt) = config.prebuilt {
                    macro_rules! file_pull {
                        ($($x:ident), *) => {
                            {
                                $(if args.$x.is_none() {
                                    args.$x = prebuilt.$x;
                                })*
                            }
                        };
                    }
                    macro_rules! file_pull_switch {
                        ($($x:ident), *) => {
                            {
                                $(if !args.$x {
                                    if let Some(opt) = prebuilt.$x {
                                        args.$x = opt;
                                    }
                                })*
                            }
                        };
                    }

                    file_pull![target, index_key, path, report_path, reports];
                    file_pull_switch![no_create_path, no_verify, safe, out, color, no_color];
                }

                match (&args.index, &args.index_key) {
                    (Some(index), None) => {
                        if let Some(cfi) = config.index {
                            for (_, i) in cfi {
                                if i.index.eq(index) {
                                    if let Some(pk) = i.pub_key {
                                        for pk in pk {
                                            args.pub_key.insert(pk);
                                        }
                                    }
                                    if args.auth.is_none() && i.auth.is_some() {
                                        args.auth = i.auth;
                                    }
                                }
                            }
                        }
                    }
                    (None, Some(index_key)) => {
                        if let Some(cfi) = config.index {
                            for (key, i) in cfi {
                                if key.eq(index_key) {
                                    args.index = Some(i.index);
                                    if let Some(pk) = i.pub_key {
                                        for pk in pk {
                                            args.pub_key.insert(pk);
                                        }
                                    }
                                    if args.auth.is_none() && i.auth.is_some() {
                                        args.auth = i.auth;
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Err(err) => panic!("Failed to parse config file.\n{err}"),
        }
    }
    else {
        eprintln!("WARN: Could not find config, it will be ignored.");
    }

    assert!(
        args.config.is_none(),
        "Could not find an existing config files. Maybe try to generate one using --gen-config?"
    );
    assert!(
        !args.require_config,
        "Config file required, but not found at {conf:?}. Did you mean to use --config=$PATH?"
    );
}

fn convert(args: Arguments) -> Config {
    let target = args.target.unwrap_or_else(|| TARGET.to_owned());

    let index = args.index.unwrap_or_else(|| DEFAULT_INDEX.to_string());

    let auth = args.auth;

    let path = args.path.unwrap_or_else(|| {
        let mut cargo_home = cargo_home().expect("Could not find cargo home directory. Please set $CARGO_HOME, or use $PREBUILT_PATH or --path");
        if !cargo_home.ends_with("bin") {
            cargo_home.push("bin");
        }
        cargo_home
    });

    let report_path = args.report_path.unwrap_or_else(|| {
        ProjectDirs::from(QUALIFIER, ORG, APPLICATION).map_or_else(
            || panic!("Could not get report path, try setting $XDG_DATA_HOME or $HOME."),
            |project| {
                let mut data = PathBuf::from(project.data_dir());
                data.push("reports");
                data
            },
        )
    });

    let ci = args.ci;
    let no_create_path = args.no_create_path;

    let reports = args
        .reports
        .unwrap_or_else(|| IndexSet::from([ReportType::LicenseDL]));

    let no_verify = args.no_verify;
    let safe = args.safe;
    let out = args.out;
    let get_latest = args.get_latest;

    let sigs = args.pub_key;

    match (args.color, args.no_color) {
        (true, false) => color::set_override(true),
        (_, true) => color::set_override(false),
        _ => {}
    }

    let pkgs = args.pkgs;

    Config {
        target,
        index,
        auth,
        path,
        report_path,
        ci,
        no_create_path,
        reports,
        sigs,
        no_verify,
        safe,
        out,
        get_latest,
        pkgs,
    }
}

pub fn get() -> Config {
    // arguments and env vars
    let mut args = parse_args();
    #[cfg(debug_assertions)]
    dbg!(&args);

    // Check 1
    // --index and --index-key conflict
    assert!(
        !(args.index.is_some() && args.index_key.is_some()),
        "Arguments {} and {} {}.",
        err_color_print("--index", &PossibleColor::BrightBlue),
        err_color_print("--index-key", &PossibleColor::BrightBlue),
        err_color_print("conflict", &PossibleColor::BrightRed),
    );

    // Load from config file
    if !args.ci {
        fill_from_file(&mut args);
        #[cfg(debug_assertions)]
        dbg!(&args);
    }

    // Check 2
    // Check index and add cargo-prebuilt-index pub key if needed.
    if args.index.is_none() || args.index.as_ref().unwrap().eq(DEFAULT_INDEX) {
        args.pub_key
            .insert(include_str!("../keys/cargo-prebuilt-index.pub").to_string());
    }

    convert(args)
}

#[cfg(test)]
mod test {
    use minisign_verify::{PublicKey, Signature};

    #[test]
    fn test_minisign1() {
        let data = include_bytes!("../test/pubdata.test");
        let sig = include_str!("../test/pubdata.test.minisig");
        let pubkey = include_str!("../test/pubdata.pub");

        let signature = Signature::decode(sig).unwrap();
        let pk = PublicKey::from_base64(pubkey).unwrap();
        pk.verify(data, &signature, false).unwrap();
    }
}
