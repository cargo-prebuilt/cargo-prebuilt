mod args;
mod conf_file;
mod get;
mod interact;
#[cfg(test)]
mod test;

use flate2::read::GzDecoder;
use home::cargo_home;
use owo_colors::{OwoColorize, Stream::Stdout};
use sha2::{Digest, Sha256};
use std::{env, fs, fs::create_dir_all, path::Path, str, string::ToString, sync::Arc};
use tar::Archive;

static TARGET: &str = env!("TARGET");

fn main() -> Result<(), String> {
    // Bypass bpaf, print version, then exit.
    let args: Vec<String> = env::args().collect();
    if args.contains(&"--version".to_string()) || args.contains(&"-v".to_string()) {
        println!(env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    let mut args = args::parse_args();
    #[cfg(debug_assertions)]
    dbg!(&args);

    // Try to get index from config file.
    if !args.ci && args.index.is_none() {
        args.index = conf_file::get_index();
    }

    let args = args;

    if args.colors {
        owo_colors::set_override(true);
    }
    if args.no_colors {
        owo_colors::set_override(false);
    }

    let target = args.target.as_str();

    let prebuilt_bin = args.path.clone().unwrap_or_else(|| {
        let mut cargo_home = cargo_home().expect("Could not find cargo home directory, please set CARGO_HOME or PREBUILT_PATH, or use --path");
        if !cargo_home.ends_with("bin") {
            cargo_home.push("bin");
        }
        cargo_home
    });

    if !args.no_create_path && create_dir_all(&prebuilt_bin).is_err() {
        println!("Could not create the directories {prebuilt_bin:?}.");
        std::process::exit(-44);
    }
    else if !Path::new(&prebuilt_bin).exists() {
        println!("Directories do not exist! {prebuilt_bin:?}.");
        std::process::exit(-45);
    }

    // Build ureq agent
    let agent = ureq::AgentBuilder::new()
        .tls_connector(Arc::new(
            native_tls::TlsConnector::new().expect("Could not create TlsConnector"),
        ))
        .https_only(true)
        .user_agent(format!("cargo-prebuilt_cli {}", env!("CARGO_PKG_VERSION")).as_str())
        .build();

    let interact = interact::create_interact(&args.index, &args.auth, agent);
    let interact = interact.as_ref();

    // Get pkgs
    let pkgs: Vec<&str> = args.pkgs.split(',').collect();
    for pkg in pkgs {
        let mut id = pkg;
        let mut version = None; // None will pull the latest version

        // If there is a version string get it
        if let Some((i, j)) = id.split_once('@') {
            id = i;
            version = Some(j)
        }

        // Get latest version
        let version = match version {
            Some(v) => v.to_string(),
            None => get::latest_version(interact, id),
        };
        let version = version.as_str();

        // Download hash
        let sha_hash = get::hash(interact, id, version, target);

        // Download tarball
        let tar_bytes = get::tar(interact, id, version, target);

        // Check hash
        let mut hasher = Sha256::new();
        hasher.update(&tar_bytes);
        let hash: Vec<u8> = hasher.finalize().to_vec();
        let hash = hex::encode(hash);

        if !(hash.eq(&sha_hash)) {
            println!("Hashes do not match.");
            std::process::exit(-256);
        }

        // Extract Tar
        let reader = std::io::Cursor::new(tar_bytes);
        let mut archive = Archive::new(GzDecoder::new(reader));
        match archive.entries() {
            Ok(es) => {
                println!(
                    "{} {id} {version}...",
                    "Extracting".if_supports_color(Stdout, |text| text.bright_blue())
                );

                for e in es {
                    let mut e = e.expect("Malformed entry.");

                    let mut path = prebuilt_bin.clone();
                    path.push(e.path().expect("Could not extract path from tar."));

                    e.unpack(&path)
                        .expect("Could not extract binaries from downloaded tar archive");

                    let abs = fs::canonicalize(path).expect("Could not canonicalize install path");
                    println!(
                        "{} {abs:?}.",
                        "Added".if_supports_color(Stdout, |text| text.bright_purple())
                    );
                }
            }
            Err(_) => {
                println!("Cannot get entries from downloaded tar.");
                std::process::exit(-13);
            }
        }

        // Reports
        get::reports(interact, &args, &prebuilt_bin, id, version);

        println!(
            "{} {id} {version}.",
            "Installed".if_supports_color(Stdout, |text| text.bright_green())
        );
    }

    println!("{}", "Done!".if_supports_color(Stdout, |text| text.green()));

    Ok(())
}
