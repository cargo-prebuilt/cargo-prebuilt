use crate::{
    color::{self, err_color_print, PossibleColor},
    data::{ConfigFileKeysV1, ConfigFilePrebuiltV1, ConfigFileV1, ReportType, SigKeys},
    APPLICATION, DEFAULT_INDEX, ORG, QUALIFIER, TARGET,
};
use bpaf::*;
use directories::ProjectDirs;
use home::cargo_home;
use indexmap::IndexSet;
use std::{
    collections::HashMap,
    fs::{create_dir_all, File, OpenOptions},
    io::{Read, Seek, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

static CONFIG_FILE: &str = "config.toml";

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
    pub sigs: Vec<String>,
    pub no_verify: bool,
    pub safe: bool,
    pub out: bool,
    pub get_latest: bool,
    pub pkgs: IndexSet<String>,
}

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
    pub_key: Option<String>,
    no_verify: bool,
    safe: bool,
    out: bool,
    color: bool,
    no_color: bool,
    gen_config: bool,
    get_latest: bool,
    pkgs: IndexSet<String>,
}

fn parse_args() -> Arguments {
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
        .help("Target of the binary to download. (Defaults to target of cargo-prebuilt)")
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
        .help("Path to the config file")
        .argument::<PathBuf>("CONFIG_PATH")
        .optional();

    let path = long("path")
        .env("PREBUILT_PATH")
        .help("Path to the folder where downloaded binaries will be installed. (Default: $CARGO_HOME)")
        .argument::<PathBuf>("PATH")
        .optional();

    let report_path = long("report-path")
        .env("PREBUILT_REPORT_PATH")
        .help("Path to the folder where the reports will be put.")
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
        .help("A CSV list of reports types. (license, deps, audit)")
        .argument::<String>("REPORTS")
        .parse(|s| {
            let mut v = IndexSet::new();
            if !s.eq("") {
                for i in s.split(',') {
                    match TryInto::<ReportType>::try_into(i) {
                        Ok(d) => {
                            let _ = v.insert(d);
                        }
                        Err(_) => return Err(format!("{i} is not a report type.")),
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
        .optional();

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

    let gen_config = long("gen-config")
        .help("Generate/Overwrite a base config at $CONFIG/cargo-prebuilt/config.toml. (This still requires PKGS to be filled, but they will be ignored.)")
        .switch();

    let get_latest = long("get-latest")
        .env("PREBUILT_GET_LATEST")
        .help("Get latest versions of crates in index and then exit.")
        .switch();

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
        gen_config,
        get_latest,
        pkgs,
    });

    cargo_helper("prebuilt", parser)
        .to_options()
        .version(env!("CARGO_PKG_VERSION"))
        .run()
}

fn fill_from_file(args: &mut Arguments, sig_keys: &mut SigKeys) {
    let conf = match args.config.clone() {
        Some(p) => p,
        None => match ProjectDirs::from(QUALIFIER, ORG, APPLICATION) {
            Some(project) => {
                let mut conf = PathBuf::from(project.config_dir());
                conf.push(CONFIG_FILE);
                conf
            }
            None => {
                eprintln!("Could not find config directory! Config file will be ignored.");
                return;
            }
        },
    };

    if conf.exists() {
        let mut file = File::open(conf).expect("Could not open config file.");
        let mut str = String::new();
        file.read_to_string(&mut str)
            .expect("Could not read config file.");

        let config: Result<ConfigFileV1, toml::de::Error> = toml::from_str(&str);
        match config {
            Ok(config) => {
                if let Some(mut keys) = config.key {
                    for (k, v) in keys.iter_mut() {
                        if let Some(i_key) = &args.index_key {
                            if i_key.eq(k) {
                                args.index = Some(v.index.clone());
                            }
                        }

                        if sig_keys.contains_key(&(v.index)) {
                            sig_keys
                                .get_mut(&(v.index))
                                .unwrap()
                                .push(v.pub_key.clone());
                        }
                        else {
                            sig_keys.insert(v.index.clone(), vec![v.pub_key.clone()]);
                        }
                    }
                }

                if let Some(prebuilt) = config.prebuilt {
                    macro_rules! file_convert {
                        ($($x:ident), *) => {
                            {
                                $(if args.$x.is_none() {
                                    args.$x = prebuilt.$x;
                                })*
                            }
                        };
                    }
                    macro_rules! file_convert_switch {
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

                    file_convert![target, index, auth, path, report_path, reports];
                    file_convert_switch![no_create_path, no_verify, safe, out, color, no_color];
                }
            }
            Err(err) => eprintln!("Failed to parse config file.\n{err}"),
        }
    }

    if args.config.is_some() {
        panic!("Could not find an existing config files. Maybe try to generate one using --gen-config?");
    }

    eprintln!("WARN: Could not find config, it will be ignored.");
}

fn convert(args: Arguments, mut sigs: SigKeys) -> Config {
    let target = match args.target {
        Some(val) => val,
        None => TARGET.to_owned(),
    };

    let index = match args.index {
        Some(val) => val,
        None => DEFAULT_INDEX.to_string(),
    };

    let auth = args.auth;

    let path = match args.path {
        Some(val) => val,
        None => {
            let mut cargo_home = cargo_home().expect("Could not find cargo home directory, please set CARGO_HOME or use PREBUILT_PATH or --path");
            if !cargo_home.ends_with("bin") {
                cargo_home.push("bin");
            }
            cargo_home
        }
    };

    let report_path = match args.report_path {
        Some(val) => val,
        None => match ProjectDirs::from(QUALIFIER, ORG, APPLICATION) {
            Some(project) => {
                let mut data = PathBuf::from(project.data_dir());
                data.push("reports");
                data
            }
            None => panic!("Could not get report path, try setting $HOME."),
        },
    };

    let ci = args.ci;
    let no_create_path = args.no_create_path;

    let reports = match args.reports {
        Some(val) => val,
        None => IndexSet::from([ReportType::LicenseDL]),
    };

    let no_verify = args.no_verify;
    let safe = args.safe;
    let out = args.out;
    let get_latest = args.get_latest;

    let sigs = sigs.remove(&index).unwrap_or_else(|| {
        if no_verify {
            panic!("Expected to find public key(s) for index {index}, but there was none.");
        }
        Vec::new()
    });

    match (args.color, args.no_color) {
        (true, false) => color::set_override(true),
        (_, true) => color::set_override(false),
        _ => color::from_stream(),
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

    if args.gen_config {
        generate(&args);
    }

    // Check if sig is used with index.
    if args.pub_key.is_some() && args.index.is_none() {
        panic!("--pub-key must be used with index.");
    }

    let mut keys: SigKeys = HashMap::with_capacity(1);
    keys.insert(
        DEFAULT_INDEX.to_string(),
        vec![include_str!("../keys/cargo-prebuilt-index.pub").to_string()],
    );

    // Add sig key from args
    if let Some(k) = &args.pub_key {
        keys.insert(args.index.clone().unwrap(), vec![k.clone()]);
    }

    // config file
    if !args.ci {
        fill_from_file(&mut args, &mut keys);
        #[cfg(debug_assertions)]
        dbg!(&args);
    }

    convert(args, keys)
}

fn generate(args: &Arguments) {
    color::set_override(true);
    eprintln!(
        "{} config, this will ignore package args.",
        err_color_print("Generating", PossibleColor::BrightPurple)
    );

    let conf = match args.config.clone() {
        Some(p) => p,
        None => match ProjectDirs::from(QUALIFIER, ORG, APPLICATION) {
            Some(project) => {
                let mut conf = PathBuf::from(project.config_dir());
                create_dir_all(&conf).expect("Could not create paths for config file.");
                conf.push(CONFIG_FILE);
                conf
            }
            None => panic!(
                "{} get config directory! Try using --config or $PREBUILT_CONFIG.",
                err_color_print("Could not", color::PossibleColor::BrightRed)
            ),
        },
    };
    eprintln!("Config Path: {conf:?}");

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(conf)
        .expect("Could not create/open config file.");
    let mut str = String::new();
    file.read_to_string(&mut str)
        .expect("Could not read config file.");
    let mut config: ConfigFileV1 = toml::from_str(&str).expect("Could not parse config file.");

    // Prebuilt block
    if config.prebuilt.is_none() {
        config.prebuilt = Some(ConfigFilePrebuiltV1::default())
    }

    // Index writing
    let rand = format!(
        "gen_{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to generate a random id for index addition.")
            .as_secs()
    );
    if let (Some(index), Some(pub_key)) = (&args.index, &args.pub_key) {
        if config.key.is_none() {
            config.key = Some(HashMap::with_capacity(1));
        }
        config.key.as_mut().unwrap().insert(
            rand,
            ConfigFileKeysV1 {
                index: index.clone(),
                pub_key: pub_key.clone(),
            },
        );
        eprintln!(
            "{} an index.",
            err_color_print("Added", PossibleColor::BrightMagenta)
        );
    }

    match &mut config.prebuilt {
        Some(prebuilt) => {
            // Path writing
            if let Some(item) = &args.path {
                prebuilt.path = Some(item.to_path_buf());
                eprintln!(
                    "{} a path.",
                    err_color_print("Added", PossibleColor::BrightMagenta)
                );
            }
            // Report Path writing
            if let Some(item) = &args.report_path {
                prebuilt.report_path = Some(item.to_path_buf());
                eprintln!(
                    "{} a report path.",
                    err_color_print("Added", PossibleColor::BrightMagenta)
                );
            }
            // No Create Path writing
            if args.no_create_path {
                prebuilt.no_create_path = Some(true);
                eprintln!(
                    "{} no create path.",
                    err_color_print("Added", PossibleColor::BrightMagenta)
                );
            }
            // Reports writing
            if let Some(item) = &args.reports {
                prebuilt.reports = Some(item.clone());
                eprintln!(
                    "{} reports.",
                    err_color_print("Added", PossibleColor::BrightMagenta)
                );
            }
            // No Verify writing
            if args.no_verify {
                prebuilt.no_verify = Some(true);
                eprintln!(
                    "{} no verify.",
                    err_color_print("Added", PossibleColor::BrightMagenta)
                );
            }
            // Safe writing
            if args.safe {
                prebuilt.safe = Some(true);
                eprintln!(
                    "{} safe mode.",
                    err_color_print("Added", PossibleColor::BrightMagenta)
                );
            }
            // Out writing
            if args.out {
                prebuilt.out = Some(true);
                eprintln!(
                    "{} print events.",
                    err_color_print("Added", PossibleColor::BrightMagenta)
                );
            }
            // Color
            if args.color {
                prebuilt.color = Some(true);
                eprintln!(
                    "{} color.",
                    err_color_print("Added", PossibleColor::BrightMagenta)
                );
            }
            if args.no_color {
                prebuilt.no_color = Some(true);
                eprintln!(
                    "{} no color.",
                    err_color_print("Added", PossibleColor::BrightMagenta)
                );
            }
        }
        None => panic!("How you get here?"),
    }

    // Rewind time
    file.rewind().expect("Could not rewind config file stream.");

    // Write to config
    let str = toml::to_string(&config).expect("Could not convert ConfigFile to string.");
    file.write_all(str.as_bytes())
        .expect("Could not write to config file.");

    eprintln!(
        "{}",
        err_color_print("Generated Config!", color::PossibleColor::Green)
    );
    std::process::exit(0);
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
