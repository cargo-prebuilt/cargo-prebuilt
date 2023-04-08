mod args;
mod get;
mod interact;
#[cfg(test)]
mod test;

use flate2::read::GzDecoder;
use owo_colors::{OwoColorize, Stream::Stdout};
use sha2::{Digest, Sha256};
use std::{
    env,
    ffi::OsString,
    fs,
    fs::{create_dir_all, File},
    path::PathBuf,
    process::Command,
    str,
    string::ToString,
    sync::Arc,
};
use tar::Archive;

static TARGET: &str = env!("TARGET");

fn main() -> Result<(), String> {
    // Bypass bpaf, print version, then exit.
    let args: Vec<String> = env::args().collect();
    if args.contains(&"--version".to_string()) || args.contains(&"-v".to_string()) {
        println!(env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    let args = args::parse_args();

    #[cfg(debug_assertions)]
    dbg!(&args);

    if args.colors {
        owo_colors::set_override(true);
    }
    if args.no_colors {
        owo_colors::set_override(false);
    }

    let target = args.target.as_str();

    let mut prebuilt_home = args.path.clone().unwrap_or_else(detect_cargo);
    if !args.no_bin && !prebuilt_home.ends_with("bin") {
        prebuilt_home.push("bin");
    }
    let cargo_bin = prebuilt_home;

    if !args.no_create_home && create_dir_all(&cargo_bin).is_err() {
        println!("Could not create the dirs {cargo_bin:?}.");
        std::process::exit(-44);
    }

    // Build ureq agent
    let agent = ureq::AgentBuilder::new()
        .tls_connector(Arc::new(
            native_tls::TlsConnector::new().expect("Could not create TlsConnector"),
        ))
        .https_only(true)
        .user_agent(format!("cargo-prebuilt {}", env!("CARGO_PKG_VERSION")).as_str())
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

                    let mut path = cargo_bin.clone();
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
        get::reports(interact, &args, &cargo_bin, id, version);

        println!(
            "{} {id} {version}.",
            "Installed".if_supports_color(Stdout, |text| text.bright_green())
        );
    }

    println!("{}", "Done!".if_supports_color(Stdout, |text| text.green()));

    Ok(())
}

fn detect_cargo() -> PathBuf {
    // Try to find CARGO_HOME by searching for cargo executable in common paths.
    let ext = if TARGET.contains("windows") { ".exe" } else { "" };
    let mut home_cargo_dir = home::home_dir().unwrap_or_else(|| PathBuf::from("~"));
    home_cargo_dir.push(".cargo/bin/cargo");
    for path in [
        home_cargo_dir,
        PathBuf::from(format!("/usr/local/cargo/bin/cargo{ext}")),
    ]
    .iter_mut()
    {
        if File::open(&path).is_ok() {
            let abs = fs::canonicalize(&path).expect("Could not canonicalize cargo path");
            println!("Detected cargo at {abs:?}. Will install into this folder.");

            path.pop(); // Remove cargo executable.
            if path.ends_with("bin") {
                // Remove bin if the folder is appended.
                path.pop();
            }
            return path.clone();
        }
    }

    // Try to find CARGO_HOME by using which/where.exe to get where the cargo executable is.
    #[cfg(target_family = "unix")]
    {
        use std::os::unix::ffi::OsStringExt;

        println!("WARN: Using which to find cargo and then deduce CARGO_HOME.");

        let out = Command::new("sh")
            .args(["-c", "which cargo"])
            .output()
            .expect("Could not use which to detect where cargo is.");
        if out.status.success() {
            let mut path = PathBuf::from(OsString::from_vec(out.stdout));
            path.pop(); // Remove cargo executable.
            if path.ends_with("bin") {
                // Remove bin if the folder is appended.
                path.pop();
            }
            return path;
        }

        println!("Could not detect cargo using which. Please set the CARGO_HOME env var.");
        std::process::exit(-125);
    }
    #[cfg(target_family = "windows")]
    {
        use std::os::windows::ffi::OsStringExt;

        println!("WARN: Using where.exe to find cargo and then deduce CARGO_HOME.");

        let out = Command::new("cmd")
            .args(["/C", "where.exe cargo.exe"])
            .output()
            .expect("Could not use where.exe to detect where cargo is.");
        if out.status.success() {
            let p_vec: Vec<u16> = out.stdout.iter().map(|a| *a as u16).collect();
            let mut path = PathBuf::from(OsString::from_wide(&p_vec));
            path.pop(); // Remove cargo executable.
            if path.ends_with("bin") {
                path.pop();
            } // Remove bin if the folder is appended.
            return path;
        }

        println!("Could not detect cargo using where.exe. Please set the CARGO_HOME env var.");
        std::process::exit(-126);
    }
    #[cfg(not(any(target_family = "unix", target_family = "windows")))]
    {
        println!("Platform does not support which/where.exe detection of cargo. Please set the CARGO_HOME env var.");
        std::process::exit(-122);
    }
}
