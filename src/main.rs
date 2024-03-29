#![forbid(unsafe_code)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::multiple_crate_versions)]

mod color;
mod config;
mod data;
mod events;
mod get;
mod interact;

use flate2::read::GzDecoder;
use std::{
    fs::{self, create_dir_all, File},
    io::{Read, Write},
    path::Path,
    str,
};
use tar::Archive;

use crate::{
    color::{err_color_print, PossibleColor},
    config::Config,
    data::InfoFileImm,
    get::Fetcher,
};

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

static QUALIFIER: &str = "tech";
static ORG: &str = "harmless";
static APPLICATION: &str = "cargo-prebuilt";

static DEFAULT_INDEX: &str = "gh-pub:github.com/cargo-prebuilt/index";
static TARGET: &str = env!("TARGET");

fn main() {
    let config = config::get();
    let config = &config;
    #[cfg(debug_assertions)]
    dbg!(&config);

    // Check if a needed feature was excluded.
    should_error();

    if !config.no_create_path && create_dir_all(&config.path).is_err() {
        panic!("Could not create the directory '{:?}'.", config.path);
    }
    else if !Path::new(&config.path).exists() {
        panic!("Directory does not exist! '{:?}'.", config.path);
    }

    // Only create/check reports path if needed.
    if !config.ci && !config.reports.is_empty() {
        if !config.no_create_path && create_dir_all(&config.report_path).is_err() {
            panic!("Could not create the directory '{:?}'.", config.report_path);
        }
        else if !Path::new(&config.report_path).exists() {
            panic!("Directory does not exist! '{:?}'.", config.report_path);
        }
    }

    // Build ureq agent
    let agent = create_agent();

    // Create Fetcher which is used to fetch items from index.
    let mut fetcher = Fetcher::new(config, agent);

    // Get pkgs
    for pkg in &config.pkgs {
        let mut id = pkg.as_str();
        let mut version = None; // None will pull the latest version

        // If there is a version string get it
        if let Some((i, j)) = id.split_once('@') {
            id = i;
            version = Some(j);
        }

        // If --get-latest then get latest version and print out latest event
        if config.get_latest {
            events::get_latest(id, &fetcher.get_latest(id));
            continue;
        }

        // Get the version that fetcher is using
        let version = version.map_or_else(|| fetcher.get_latest(id), ToString::to_string);
        let version = &version;

        events::target(id, version, config);

        // Download and hash tar
        let (info, _hashes, tar_bytes) = fetcher.download(id, version, config);
        let info = &info;

        // Extract Tar
        extract(info, config, id, version, tar_bytes);

        // Reports
        if !config.ci {
            fetcher.reports(id, version, info, config);
        }

        eprintln!(
            "{} {id}@{version}.",
            err_color_print("Installed", &PossibleColor::BrightGreen)
        );
        events::installed(id, version, config);
    }

    eprintln!("{}", err_color_print("Done!", &PossibleColor::Green));
}

fn extract(info: &InfoFileImm, config: &Config, id: &str, version: &str, tar_bytes: Vec<u8>) {
    let reader = std::io::Cursor::new(tar_bytes);
    let mut archive = Archive::new(GzDecoder::new(reader));

    let es = archive
        .entries()
        .expect("Cannot get entries from downloaded tar.");

    eprintln!(
        "{} {id}@{version}...",
        err_color_print("Extracting", &PossibleColor::BrightBlue)
    );

    for e in es {
        let mut e = e.expect("Malformed entry in tarball.");

        let mut blob_data = Vec::new();
        e.read_to_end(&mut blob_data)
            .expect("Could not extract binary from archive.");

        let bin_path = e.path().expect("Could not extract path from archive.");
        let str_name = bin_path
            .clone()
            .into_owned()
            .into_os_string()
            .into_string()
            .expect("Archive has non utf-8 path.");

        // Make sure there are no path separators since this will be appended
        assert!(
            !str_name.contains(std::path::is_separator),
            "{} path separator in archive for {id}@{version}",
            err_color_print("Illegal", &PossibleColor::BrightRed)
        );

        assert!(
            Fetcher::is_bin(info, &str_name),
            "{} binary ({str_name}) in archive for {id}@{version}",
            err_color_print("Illegal", &PossibleColor::BrightRed)
        );

        let mut path = config.path.clone();
        path.push(bin_path);

        assert!(
            !(config.safe && !config.ci && path.exists()),
            "Binary {str_name} {} for {id}@{version}",
            err_color_print("already exists", &PossibleColor::BrightRed)
        );

        let mut file = File::create(&path).expect("Could not open file to write binary to.");
        file.write_all(&blob_data)
            .expect("Could not write binary to file.");

        // Add +x permission on unix platforms.
        #[cfg(target_family = "unix")]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755))
                .expect("Could not set permissions.");
        }

        let abs = fs::canonicalize(path).expect("Could not canonicalize install path.");

        eprintln!(
            "{} {abs:?}.",
            err_color_print("Installed", &PossibleColor::BrightPurple)
        );

        events::binary_installed(id, version, config, abs.as_path());
    }
}

const fn should_error() {
    // No TLS
    #[cfg(not(any(feature = "native", feature = "rustls")))]
    panic!("cargo-prebuilt only supports https and was built without the 'native' or 'rustls' feature.");

    // No Indexes
    #[cfg(not(any(feature = "github-public", feature = "github-private")))]
    panic!("cargo-prebuilt was not built with any indexes, try the 'indexes' feature.");
}

fn create_agent() -> ureq::Agent {
    #[cfg(feature = "native")]
    let agent = ureq::AgentBuilder::new().tls_connector(std::sync::Arc::new(
        native_tls::TlsConnector::new().expect("Could not create TlsConnector"),
    ));

    #[cfg(feature = "rustls")]
    let agent = ureq::AgentBuilder::new();

    #[cfg(any(feature = "native", feature = "rustls"))]
    let agent = agent
        .https_only(true)
        .user_agent(format!("cargo-prebuilt_cli {}", env!("CARGO_PKG_VERSION")).as_str())
        .build();

    // Allows for any feature set to be built for, even though this is unsupported.
    #[cfg(not(any(feature = "native", feature = "rustls")))]
    let agent = ureq::agent();

    agent
}
