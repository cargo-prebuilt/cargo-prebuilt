use crate::{
    data::{ConfigFileV1, SigKeys},
    TARGET,
};
use bpaf::*;
use home::{cargo_home, home_dir};
use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

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
    pub hashes: Option<String>, // Use by priority if None. (sha3_512 -> sha3_256 -> sha512 -> sha256)
    pub sigs: SigKeys,
    pub force_sig: bool,
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
    no_create_path: bool,
    reports: Option<String>,
    hashes: Option<String>,
    sig: Option<String>,
    force_sig: bool,
    color: bool,
    no_color: bool,
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
        .env("PREBUILT_NO_CREATE_PATH")
        .help("Do not create the report and/or bin folder if it is missing.")
        .switch();

    let reports = long("reports")
        .env("PREBUILT_REPORTS")
        .help("A CSV list of reports types. (license-out, license-dl, deps-out, deps-dl, audit-out, audit-dl)")
        .argument::<String>("REPORTS")
        .optional();

    let hashes = long("hashes")
        .env("PREBUILT_HASHES")
        .help("A CSV list of hash types. (sha256, sha512, sha3_256, sha3_512)")
        .argument::<String>("HASHES")
        .optional();

    let sig = long("sig")
        .env("PREBUILT_SIG")
        .help("A public verifying key encoded as base64. Must be used with --index.")
        .argument::<String>("SIG")
        .optional();

    let force_sig = long("force-sig")
        .env("PREBUILT_FORCE_SIG")
        .help("Force verifying signatures.")
        .switch();

    let color = long("color")
        .env("PREBUILT_COLOR")
        .help("Force color to be turned on.")
        .switch();

    let no_color = long("no-color")
        .env("PREBUILT_NO_COLOR")
        .help("Force color to be turned off.")
        .switch();

    let parser = construct!(Arguments {
        target,
        index,
        auth,
        path,
        report_path,
        ci,
        no_create_path,
        reports,
        hashes,
        sig,
        force_sig,
        color,
        no_color,
        pkgs,
    });

    cargo_helper("prebuilt", parser)
        .to_options()
        .version(env!("CARGO_PKG_VERSION"))
        .run()
}

fn fill_from_file(args: &mut Arguments, sig_keys: &mut SigKeys) {
    match home_dir() {
        Some(mut conf) => {
            conf.push(".config/cargo-prebuilt/config.toml");
            if conf.exists() {
                let mut file = File::open(conf).expect("Could not open config file.");
                let mut str = String::new();
                file.read_to_string(&mut str)
                    .expect("Could not read config file.");

                let config: Result<ConfigFileV1, toml::de::Error> = toml::from_str(&str);
                match config {
                    Ok(config) => {
                        if let Some(mut keys) = config.key {
                            for (_, v) in keys.iter_mut() {
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
                            // TODO: Way to not clone?
                            macro_rules! file_convert {
                                ($($x:ident), *) => {
                                    {
                                        $(args.$x = args.$x.clone().or(prebuilt.$x);)*
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
                            macro_rules! file_convert_csv {
                                ($($x:ident), *) => {
                                    {
                                        $(args.$x = args.$x.clone().or_else(|| {
                                            prebuilt.$x.map_or_else(
                                                || None,
                                                |val| {
                                                    Some(
                                                        val.iter()
                                                            .map(|v| {
                                                                let str: &str = v.into();
                                                                String::from(str)
                                                            })
                                                            .collect::<Vec<String>>()
                                                            .join(","),
                                                    )
                                                },
                                            )
                                        });)*
                                    }
                                };
                            }

                            file_convert![target, index, auth, path, report_path];
                            file_convert_switch![no_create_path, force_sig, color];
                            file_convert_csv![reports, hashes];
                        }
                    }
                    Err(err) => eprintln!("Failed to parse config file.\n{err}"),
                }
            }
        }
        None => eprintln!("Could not find home directory! Config file will be ignored."),
    }
}

fn convert(args: Arguments, sigs: SigKeys) -> Config {
    let target = match args.target {
        Some(val) => val,
        None => TARGET.to_owned(),
    };

    let index = match args.index {
        Some(val) => val,
        None => "gh-pub:github.com/cargo-prebuilt/index".to_string(),
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
        None => {
            let mut prebuilt_home = home_dir().expect("Could not find home directory, please set HOME or use PREBUILT_REPORT_PATH or --report-path");
            prebuilt_home.push(".prebuilt");
            prebuilt_home
        }
    };

    let ci = args.ci;

    let no_create_path = args.no_create_path;

    let reports = match args.reports {
        Some(val) => val,
        None => REPORT_FLAGS[1].to_owned(),
    };

    let hashes = args.hashes;

    let force_sig = args.force_sig;

    match (args.color, args.no_color) {
        #[cfg(any(feature = "bright-color", feature = "dull-color"))]
        (true, false) => owo_colors::set_override(true),
        (_, true) => owo_colors::set_override(false),
        _ => {}
    }

    #[cfg(not(any(feature = "bright-color", feature = "dull-color")))]
    owo_colors::set_override(false);

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
        hashes,
        sigs,
        force_sig,
        pkgs,
    }
}

pub fn get() -> Config {
    // arguments and env vars
    let mut args = parse_args();
    #[cfg(debug_assertions)]
    dbg!(&args);

    // Check if sig is used with index.
    if args.sig.is_some() && args.index.is_none() {
        eprintln!("--sig must be used with --index.");
        std::process::exit(502);
    }

    //TODO: Load default sig key.
    let mut keys: SigKeys = HashMap::new();
    // Add sig key if needed
    if let Some(k) = &args.sig {
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

#[cfg(test)]
mod test {
    use pgp::SignedPublicKey;
    use pgp::StandaloneSignature;
    use pgp::Deserializable;

    #[test]
    fn test_pgp() {
        let data = include_bytes!("../test/pubdata.test");
        let sig = include_bytes!("../test/pubdata.test.sig");
        let pubkey = include_bytes!("../test/pubkey.asc");

        let mut reader = std::io::Cursor::new(sig);
        let sig = StandaloneSignature::from_bytes(&mut reader).unwrap();

        let mut reader = std::io::Cursor::new(pubkey);
        let pubkey = SignedPublicKey::from_armor_single(&mut reader).unwrap().0;

        sig.verify(&pubkey, data).unwrap();
    }
}
