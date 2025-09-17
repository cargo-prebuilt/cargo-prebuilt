#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![allow(clippy::multiple_crate_versions)]
#![deny(clippy::std_instead_of_core)]
// #![deny(clippy::std_instead_of_alloc)]
#![deny(clippy::alloc_instead_of_core)]

// TODO: Allow setting timeout?
// TODO: Allow retries?
// TODO: Improve errors? Make them more readable.

mod coloring;
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
    sync::Arc,
};
use tar::Archive;
use ureq::config::AutoHeaderValue;

use crate::{
    data::{InfoFileImm, Meta},
    get::Fetcher,
};

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

static BUILD_REPO_LINK: &str = match option_env!("PREBUILT_BUILD_REPO_LINK") {
    Some(s) => s,
    None => "https://github.com/cargo-prebuilt/cargo-prebuilt",
};
static BUILD_ISSUES_LINK: &str = match option_env!("PREBUILT_BUILD_ISSUES_LINK") {
    Some(s) => s,
    None => "https://github.com/cargo-prebuilt/cargo-prebuilt/issues",
};
static BUILD_DOCS_LINK: &str = match option_env!("PREBUILT_BUILD_DOCS_LINK") {
    Some(s) => s,
    None => concat!(
        "https://github.com/cargo-prebuilt/cargo-prebuilt/tree/v",
        env!("CARGO_PKG_VERSION"),
        "/docs"
    ),
};

static QUALIFIER: &str = "tech";
static ORG: &str = "harmless";
static APPLICATION: &str = "cargo-prebuilt";

static DEFAULT_INDEX: &str = match option_env!("PREBUILT_BUILD_DEFAULT_INDEX") {
    Some(s) => s,
    None => "gh-pub:github.com/cargo-prebuilt/index",
};
static DEFAULT_INDEX_KEY: &str = match option_env!("PREBUILT_BUILD_DEFAULT_INDEX_KEY") {
    Some(s) => s,
    None => include_str!("../keys/cargo-prebuilt-index.pub"),
};
static DEFAULT_TARGET: &str = match option_env!("PREBUILT_BUILD_DEFAULT_TARGET") {
    Some(s) => s,
    None => env!("TARGET"),
};

const BLOB_LIMIT: u64 = 1_048_576 * 50; // 50 MB

fn main() {
    #[cfg(debug_assertions)]
    dbg!(
        BUILD_REPO_LINK,
        BUILD_ISSUES_LINK,
        BUILD_DOCS_LINK,
        DEFAULT_INDEX,
        DEFAULT_INDEX_KEY,
        DEFAULT_TARGET
    );

    for a in std::env::args_os() {
        if a.eq("--version") || a.eq("-V") {
            println!("Version: {}", env!("CARGO_PKG_VERSION"));
            println!("Default Target: {DEFAULT_TARGET}");
            println!("Default Index: {DEFAULT_INDEX}");
            println!("Default Index Key(s): {DEFAULT_INDEX_KEY}");
            std::process::exit(0);
        } else if a.eq("--docs") {
            println!("Repo: {BUILD_REPO_LINK}");
            println!("Issues: {BUILD_ISSUES_LINK}");
            println!("Docs: {BUILD_DOCS_LINK}");
            std::process::exit(0);
        }
    }

    let config = config::get();
    let config = &config;
    #[cfg(debug_assertions)]
    dbg!(&config);

    // Check if a needed feature was excluded.
    should_error();

    if !config.no_create_path && create_dir_all(&config.path).is_err() {
        panic!(
            "Could not create the directory '{}'.",
            config.path.display()
        );
    } else if !Path::new(&config.path).exists() {
        panic!("Directory does not exist! '{}'.", config.path.display());
    }

    // Only create/check reports path if needed.
    if !config.ci && !config.reports.is_empty() {
        if !config.no_create_path && create_dir_all(&config.report_path).is_err() {
            panic!(
                "Could not create the directory '{}'.",
                config.report_path.display()
            );
        } else if !Path::new(&config.report_path).exists() {
            panic!(
                "Directory does not exist! '{}'.",
                config.report_path.display()
            );
        }
    }

    // Build ureq agent
    let agent = create_agent();

    // Create Fetcher which is used to fetch items from index.
    let mut fetcher = Fetcher::new(config, agent);

    // Get pkgs
    for pkg in &config.packages {
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

        let meta = Meta::new(id, version, config);
        let meta = &meta;

        events::target(meta);

        // Download and hash tar
        let info = fetcher.download_info(meta);
        let info = &info;

        // Check to update or not
        if config.update && !should_update(meta, info) {
            eprintln!(
                "{} for {id}@{version}. Already up to date.",
                color!(magenta, "No Change")
            );
            events::no_update(meta);
            continue;
        }

        let tar_bytes = fetcher.download_blob(meta, info);

        // Extract Tar
        extract(meta, info, tar_bytes);

        // Reports
        if !config.ci {
            fetcher.reports(meta, info);
        }

        eprintln!("{} {id}@{version}.", color!(bright_green, "Installed"));
        events::installed(meta);
    }

    eprintln!("{}", color!(green, "Done!"));
}

fn should_update(meta: &Meta, info: &InfoFileImm) -> bool {
    let mut should_update = true;

    for bin in &info.bins {
        if let Some(hashes) = info.bins_hashes.get(bin) {
            let mut bin_name = bin.clone();
            if meta.config.target.contains("windows") {
                bin_name.push_str(".exe");
            }

            let mut path = meta.config.path.clone();
            path.push(&bin_name);

            if let Ok(bytes) = fs::read(&path) {
                if Fetcher::verify_bytes_update(hashes, bin, &bytes) {
                    should_update = false;
                    continue;
                }

                eprintln!(
                    "{} for {}@{}. Hashes do not match.",
                    color!(magenta, "Will Update"),
                    meta.id,
                    meta.version,
                );
                should_update = true;
                break;
            }

            eprintln!(
                "{} for {}@{}. Cannot find/open binary '{}'.",
                color!(magenta, "Will Update"),
                meta.id,
                meta.version,
                bin_name,
            );
            should_update = true;
            break;
        }

        eprintln!(
            "{} for {}@{}. Missing binary hash.",
            color!(magenta, "Will Update"),
            meta.id,
            meta.version,
        );
        should_update = true;
        break;
    }

    should_update
}

fn extract(meta: &Meta, info: &InfoFileImm, tar_bytes: Vec<u8>) {
    let reader = std::io::Cursor::new(tar_bytes);
    let mut archive = Archive::new(GzDecoder::new(reader));

    let es = archive
        .entries()
        .expect("Cannot get entries from downloaded tar.");

    eprintln!(
        "{} {}@{}...",
        color!(bright_blue, "Extracting"),
        meta.id,
        meta.version
    );

    for e in es {
        let mut e = e.expect("Malformed entry in tarball.");

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
            "{} path separator in archive for {}@{}",
            color!(bright_red, "Illegal"),
            meta.id,
            meta.version
        );

        assert!(
            Fetcher::is_bin(info, &str_name),
            "{} binary ({str_name}) in archive for {}@{}",
            color!(bright_red, "Illegal"),
            meta.id,
            meta.version
        );

        let mut path = meta.config.path.clone();
        path.push(bin_path);

        let mut blob_data = Vec::new();
        e.read_to_end(&mut blob_data)
            .expect("Could not extract binary from archive.");

        if meta.config.hash_bins {
            Fetcher::verify_binary(meta, info, &str_name, &blob_data);
        }

        let mut file = File::create(&path).expect("Could not open file to write binary to.");
        file.write_all(&blob_data)
            .expect("Could not write binary to file.");

        // Attempt to add +x permission on unix platforms.
        #[cfg(target_family = "unix")]
        {
            use std::os::unix::fs::PermissionsExt;
            if fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).is_err() {
                eprintln!(
                    "Could not set mode 755 for {}@{} binary {str_name}",
                    meta.id, meta.version
                );
            }
        }

        let abs = dunce::canonicalize(path).expect("Could not canonicalize install path.");

        eprintln!("{} {}", color!(bright_purple, "Installed"), abs.display());

        events::binary_installed(meta, &abs.display().to_string());
    }
}

const fn should_error() {
    // No TLS
    #[cfg(not(any(feature = "native", feature = "rustls")))]
    panic!(
        "cargo-prebuilt only supports https and was built without the 'native' or 'rustls' feature."
    );

    // No Indexes
    #[cfg(not(any(feature = "github-public", feature = "github-private")))]
    panic!("cargo-prebuilt was not built with any indexes, try the 'indexes' feature.");
}

fn create_agent() -> ureq::Agent {
    #[cfg(feature = "native")]
    #[allow(unused_variables)]
    let agent = {
        use ureq::tls::{TlsConfig, TlsProvider};
        ureq::Agent::config_builder().tls_config(
            TlsConfig::builder()
                .provider(TlsProvider::NativeTls)
                .build(),
        )
    };

    #[cfg(feature = "rustls")]
    let agent = {
        use ureq::tls::{TlsConfig, TlsProvider};
        ureq::Agent::config_builder()
            .tls_config(TlsConfig::builder().provider(TlsProvider::Rustls).build())
    };

    #[cfg(any(feature = "native", feature = "rustls"))]
    let agent = agent
        .https_only(true)
        .user_agent(AutoHeaderValue::Provided(Arc::new(format!(
            "cargo-prebuilt_cli {}",
            env!("CARGO_PKG_VERSION")
        ))))
        .build();

    // Allows for any feature set to be built for, even though this is unsupported.
    #[cfg(not(any(feature = "native", feature = "rustls")))]
    let agent = ureq::agent();

    agent.into()
}
