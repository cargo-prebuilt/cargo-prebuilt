mod config;
mod data;
mod get;
mod interact;

use flate2::read::GzDecoder;
use owo_colors::{OwoColorize, Stream::Stderr};
use std::{env, fs, fs::create_dir_all, path::Path, str, string::ToString};
use tar::Archive;

use crate::get::Fetcher;

static TARGET: &str = env!("TARGET");

fn main() -> Result<(), String> {
    should_error();

    let config = config::get();
    #[cfg(debug_assertions)]
    dbg!(&config);

    let target = config.target.as_str();

    let prebuilt_bin = config.path.clone();
    if !config.no_create_path && create_dir_all(&prebuilt_bin).is_err() {
        eprintln!("Could not create the directories {prebuilt_bin:?}.");
        std::process::exit(44);
    }
    else if !Path::new(&prebuilt_bin).exists() {
        eprintln!("Directories do not exist! {prebuilt_bin:?}.");
        std::process::exit(45);
    }

    let prebuilt_home = config.report_path.clone();
    if !config.no_create_path && create_dir_all(&prebuilt_home).is_err() {
        eprintln!("Could not create the directories {prebuilt_home:?}.");
        std::process::exit(44);
    }
    else if !Path::new(&prebuilt_home).exists() {
        eprintln!("Directories do not exist! {prebuilt_home:?}.");
        std::process::exit(45);
    }

    // Build ureq agent
    let agent = create_agent();

    // Create Fetcher which is used to fetch items from index.
    let mut fetcher = Fetcher::new(&config, agent);

    // Get pkgs
    let pkgs: Vec<&str> = config.pkgs.split(',').collect();
    for pkg in pkgs {
        let mut id = pkg;
        let mut version = None; // None will pull the latest version

        // If there is a version string get it
        if let Some((i, j)) = id.split_once('@') {
            id = i;
            version = Some(j)
        }

        // Init fetcher for this crate and get latest version if needed
        fetcher.load(id, version);

        // Get version that fetcher is using
        let version = fetcher.get_version();

        // Download and hash tar
        let tar_bytes = fetcher.download(&config);

        // Extract Tar
        let reader = std::io::Cursor::new(tar_bytes);
        let mut archive = Archive::new(GzDecoder::new(reader));
        match archive.entries() {
            Ok(es) => {
                eprintln!(
                    "{} {id}@{version}...",
                    "Extracting".if_supports_color(Stderr, |text| text.bright_blue())
                );

                for e in es {
                    let mut e = e.expect("Malformed entry.");

                    let mut path = prebuilt_bin.clone();
                    path.push(e.path().expect("Could not extract path from tar."));

                    e.unpack(&path)
                        .expect("Could not extract binaries from downloaded tar archive");

                    let abs = fs::canonicalize(path).expect("Could not canonicalize install path");

                    eprintln!(
                        "{} {abs:?}.",
                        "Added".if_supports_color(Stderr, |text| text.bright_purple())
                    );

                    // Print paths to stdout too, maybe so others can parse?
                    println!("{abs:?}");
                }
            }
            Err(_) => {
                eprintln!("Cannot get entries from downloaded tar.");
                std::process::exit(13);
            }
        }

        // Reports
        //TODO: Enable
        if !config.ci {
            // fetcher.reports(&config);
        }

        eprintln!(
            "{} {id}@{version}.",
            "Installed".if_supports_color(Stderr, |text| text.bright_green())
        );

        // Prepare for next crate.
        fetcher.reset();
    }

    eprintln!("{}", "Done!".if_supports_color(Stderr, |text| text.green()));

    Ok(())
}

fn should_error() {
    // Errors
    #[cfg(not(any(feature = "native", feature = "rustls")))]
    {
        eprintln!("cargo-prebuilt only supports https and was built without the 'native' or 'rustls' feature.");
        std::process::exit(400);
    }
    #[cfg(not(any(feature = "github-public", feature = "github-private")))]
    {
        eprintln!("cargo-prebuilt was not built with any indexes, try the 'indexes' feature.");
        std::process::exit(222);
    }
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
