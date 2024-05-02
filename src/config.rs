use crate::{
    color,
    data::{ConfigFile, ReportType},
    APPLICATION, DEFAULT_INDEX, DEFAULT_INDEX_KEY, DEFAULT_TARGET, ORG, QUALIFIER,
};
use bpaf::Bpaf;
use directories::ProjectDirs;
use home::cargo_home;
use indexmap::IndexSet;
use std::{collections::HashSet, fs::File, io::Read, path::PathBuf};

static CONFIG_FILE: &str = "config.toml";

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug)]
pub struct Config {
    pub target: String,
    pub safe: bool,
    pub update: bool,
    pub index: String,
    pub pub_keys: HashSet<String>,
    pub auth: Option<String>,
    pub ci: bool,
    pub no_sig: bool,
    pub no_hash: bool,
    pub hash_bins: bool,
    pub path: PathBuf,
    pub report_path: PathBuf,
    pub no_create_path: bool,
    pub reports: IndexSet<ReportType>,
    pub out: bool,
    pub get_latest: bool,
    pub packages: IndexSet<String>,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Bpaf)]
#[bpaf(options("prebuilt"), generate(parse_args))]
struct Arguments {
    /// Target of the binary to download.
    #[bpaf(long("target"), env("PREBUILT_TARGET"), argument::<String>("TARGET"))]
    target: Option<String>,
    /// Do not overwrite binaries that already exist.
    #[bpaf(short('s'), long("safe"), env("PREBUILT_SAFE"))]
    safe: bool,
    /// Update packages based on binary hash.
    #[bpaf(short('u'), long("update"), env("PREBUILT_UPDATE"))]
    update: bool,
    /// Index to use.
    #[bpaf(long("index"), env("PREBUILT_INDEX"), argument::<String>("INDEX"))]
    index: Option<String>,
    /// A public verifying key encoded as base64. (Or a list of them using CSV). Must be used with --index.
    #[bpaf(long("pub-key"), env("PREBUILT_PUB_KEY"), argument::<String>("PUB_KEYS"), optional, parse(parse_pub_keys))]
    pub_key: HashSet<String>,
    /// Auth token to use for private indexes.
    #[bpaf(long("auth"), env("PREBUILT_AUTH"), argument::<String>("TOKEN"))]
    auth: Option<String>,
    /// Index to use, pulling from config file. Overrides --index.
    #[bpaf(long("index-key"), env("PREBUILT_INDEX_KEY"), argument::<String>("INDEX_KEY"))]
    index_key: Option<String>,
    /// Do not download reports, check for a config file, and ignore safe mode.
    #[bpaf(long("ci"), env("PREBUILT_CI"))]
    ci: bool,
    /// Do not verify downloaded info.json's and hashes.json's.
    #[bpaf(long("no-sig"), env("PREBUILT_NO_SIG"))]
    no_sig: bool,
    /// Do not verify downloaded archives.
    #[bpaf(long("no-hash"), env("PREBUILT_NO_HASH"))]
    no_hash: bool,
    /// Hash and verify extracted binaries.
    #[bpaf(long("hash-bins"), env("PREBUILT_HASH_BINS"))]
    hash_bins: bool,
    /// Path to the folder where downloaded binaries will be installed. (Default: `$CARGO_HOME/bin`)
    #[bpaf(long("path"), env("PREBUILT_PATH"), argument::<PathBuf>("PATH"))]
    path: Option<PathBuf>,
    /// Path to the folder where the reports will be put (Default: See `--docs/PATHS.md#reports`)
    #[bpaf(long("report-path"), env("PREBUILT_REPORT_PATH"), argument::<PathBuf>("REPORT_PATH"))]
    report_path: Option<PathBuf>,
    /// Do not create the report and/or bin folder if it is missing.
    #[bpaf(long("no-create-path"), env("PREBUILT_NO_CREATE_PATH"))]
    no_create_path: bool,
    /// Reports to be downloaded in a CSV format (Default: license) (See: See `--docs/REPORT_TYPES.md`)
    #[bpaf(long("reports"), env("PREBUILT_REPORTS"), argument::<String>("REPORTS"), optional, parse(parse_reports))]
    reports: Option<IndexSet<ReportType>>,
    /// Path to the config file (Default: See `--docs/PATHS.md#config`)
    #[bpaf(long("config"), env("PREBUILT_CONFIG"), argument::<PathBuf>("CONFIG_PATH"))]
    config: Option<PathBuf>,
    /// Require a config file to be used. (--ci will override this)
    #[bpaf(short('r'), long("require-config"), env("PREBUILT_REQUIRE_CONFIG"))]
    require_config: bool,
    /// Output events.
    #[bpaf(long("out"), env("PREBUILT_OUT"))]
    out: bool,
    /// Get latest versions of crates in index and then exit.
    #[bpaf(long("get-latest"), env("PREBUILT_GET_LATEST"))]
    get_latest: bool,
    /// Force color to be turned on.
    #[bpaf(long("color"), env("FORCE_COLOR"))]
    color: bool,
    /// Force color to be turned off.
    #[bpaf(long("no-color"), env("NO_COLOR"))]
    no_color: bool,
    /// Prints version information.
    #[bpaf(short('V'), long("version"))]
    #[allow(dead_code)]
    false_version: bool,
    /// Prints link to documentation.
    #[bpaf(long("docs"))]
    #[allow(dead_code)]
    false_docs: bool,
    /// A CSV list of packages with optional @VERSION.
    #[bpaf(positional::<String>("PKGS"), parse(parse_packages))]
    packages: IndexSet<String>,
}

#[allow(clippy::unnecessary_wraps)]
fn parse_pub_keys(s: Option<String>) -> Result<HashSet<String>, String> {
    Ok(s.map_or_else(HashSet::new, |s| {
        s.split(',')
            .map(std::borrow::ToOwned::to_owned)
            .collect::<HashSet<_>>()
    }))
}

fn parse_reports(s: Option<String>) -> Result<Option<IndexSet<ReportType>>, String> {
    match s {
        Some(s) => {
            let mut v = IndexSet::new();
            for i in s.split(',') {
                match TryInto::<ReportType>::try_into(i) {
                    Ok(d) => {
                        let _ = v.insert(d);
                    }
                    Err(()) => return Err(format!("{i} is not a report type.")),
                }
            }
            Ok(Some(v))
        }
        None => Ok(None),
    }
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn parse_packages(s: String) -> Result<IndexSet<String>, String> {
    let mut v = IndexSet::new();
    for i in s.split(',') {
        v.insert(i.to_string());
    }
    Ok::<IndexSet<String>, String>(v)
}

#[allow(clippy::cognitive_complexity)]
#[allow(clippy::too_many_lines)]
fn fill_from_file(args: &mut Arguments) {
    let conf = if let Some(p) = args.config.clone() {
        p
    }
    else if let Some(project) = ProjectDirs::from(QUALIFIER, ORG, APPLICATION) {
        let mut conf = PathBuf::from(project.config_dir());
        conf.push(CONFIG_FILE);
        conf
    }
    else {
        eprintln!("Could not find default config directory! Config file will be ignored.");
        return;
    };

    if conf.exists() {
        let mut file = File::open(&conf).expect("Could not open config file.");
        let mut str = String::new();
        file.read_to_string(&mut str)
            .expect("Could not read config file.");

        let config: Result<ConfigFile, _> = basic_toml::from_str(&str);
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
                    file_pull_switch![
                        safe,
                        no_sig,
                        no_hash,
                        hash_bins,
                        no_create_path,
                        out,
                        color,
                        no_color
                    ];
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
        "Could not find an existing config file."
    );
    assert!(
        !args.require_config,
        "Config file required, but not found at {conf:?}. Did you mean to use --config=$PATH?"
    );
}

fn convert(args: Arguments) -> Config {
    let target = args.target.unwrap_or_else(|| DEFAULT_TARGET.to_owned());
    let safe = args.safe;
    let update = args.update;
    let index = args.index.unwrap_or_else(|| DEFAULT_INDEX.to_string());
    let pub_keys = args.pub_key;
    let auth = args.auth;
    let ci = args.ci;
    let no_sig = args.no_sig;
    let no_hash = args.no_hash;
    let hash_bins = args.hash_bins;

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

    let no_create_path = args.no_create_path;

    let reports = args
        .reports
        .unwrap_or_else(|| IndexSet::from([ReportType::LicenseDL]));

    let out = args.out;
    let get_latest = args.get_latest;

    match (args.color, args.no_color) {
        (true, false) => color::set_override(true),
        (_, true) => color::set_override(false),
        _ => {}
    }

    let packages = args.packages;

    Config {
        target,
        safe,
        update,
        index,
        pub_keys,
        auth,
        ci,
        no_sig,
        no_hash,
        hash_bins,
        path,
        report_path,
        no_create_path,
        reports,
        out,
        get_latest,
        packages,
    }
}

pub fn get() -> Config {
    // arguments and env vars
    let mut args = parse_args().run();
    #[cfg(debug_assertions)]
    dbg!(&args);

    // TODO: When update is impl, remove this!
    assert!(
        !args.update,
        "-u/--update is currently unimplemented. It will come in a future update."
    );
    //

    // Load from config file
    if !args.ci {
        fill_from_file(&mut args);
        #[cfg(debug_assertions)]
        dbg!(&args);
    }

    // Check 1
    // Check index and add cargo-prebuilt-index pub key if needed.
    if args.index.is_none() || args.index.as_ref().unwrap().eq(DEFAULT_INDEX) {
        for s in DEFAULT_INDEX_KEY.split(',') {
            args.pub_key.insert(s.to_string());
        }
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
