mod args;
mod interact;
#[cfg(test)]
mod test;

use crate::interact::{Interact, InteractError};
use flate2::read::GzDecoder;
use owo_colors::{OwoColorize, Stream::Stdout};
use sha2::{Digest, Sha256};
use std::{
    env,
    ffi::OsString,
    fs,
    fs::{create_dir_all, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
    str,
    string::ToString,
    sync::Arc,
};
use tar::Archive;
use ureq::{Agent, Error};

static TARGET: &str = env!("TARGET");
static REPORT_FLAGS: [&str; 6] = [
    "license-out",
    "license-dl",
    "deps-out",
    "deps-dl",
    "audit-out",
    "audit-dl",
];

fn main() -> Result<(), String> {
    // Bypass bpaf, print version, then exit.
    let args: Vec<String> = env::args().collect();
    if args.contains(&"--version".to_string()) || args.contains(&"-v".to_string()) {
        println!(env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    let args = args::parse_args();

    if args.colors {
        owo_colors::set_override(true);
    }
    if args.no_colors {
        owo_colors::set_override(false);
    }

    //TODO: Allow for custom indexes
    let interact = interact::create_interact(None, None);

    let target = args.target;

    let mut prebuilt_home = args.path.unwrap_or_else(detect_cargo);
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

    // Get pkgs
    let pkgs: Vec<String> = args.pkgs.split(',').map(|s| s.to_string()).collect();

    for pkg in pkgs {
        let mut id = pkg;
        let mut version: Option<String> = None; // None will pull the latest version

        // If there is a version string get it
        if let Some((i, j)) = id.clone().split_once('@') {
            id = i.to_string();
            version = Some(j.to_string())
        }

        // Get latest version
        if version.is_none() {
            let res = match interact.get_latest(&agent, &id) {
                Ok(s) => s,
                Err(InteractError::Malformed) => {
                    println!("The version string for {id} is malformed.");
                    std::process::exit(-342);
                }
                Err(InteractError::HttpCode(404)) => {
                    println!(
                        "Crate {id} {} in index!",
                        "not found".if_supports_color(Stdout, |text| text.bright_red())
                    );
                    std::process::exit(-3);
                }
                Err(InteractError::HttpCode(code)) => {
                    println!("Http error {code} for crate {id}.");
                    std::process::exit(-4);
                }
                Err(_) => {
                    println!("Connection error.");
                    std::process::exit(-5);
                }
            };

            version = Some(res);
        }
        let version = version.unwrap();

        // Download hash
        let sha_hash = match interact.get_hash(&agent, &id, &version, &target) {
            Ok(s) => s,
            Err(InteractError::Malformed) => {
                println!("The hash string for {id} is malformed.");
                std::process::exit(-343);
            }
            Err(InteractError::HttpCode(404)) => {
                println!(
                    "Crate {id}, version {version}, and target {target} was {}! (Hash)",
                    "not found".if_supports_color(Stdout, |text| text.bright_red())
                );
                std::process::exit(-9);
            }
            Err(InteractError::HttpCode(code)) => {
                println!("Http error {code} for crate {id}.");
                std::process::exit(-10);
            }
            Err(_) => {
                println!("Connection error.");
                std::process::exit(-11);
            }
        };

        // Download tarball
        println!(
            "{} {id} {version} from {}.tar.gz",
            "Downloading".if_supports_color(Stdout, |text| text.bright_blue()),
            interact.pre_url(&id, &version, &target)
        );
        let tar_bytes = match interact.get_tar(&agent, &id, &version, &target) {
            Ok(b) => b,
            Err(InteractError::Malformed) => {
                println!("The tar bytes for {id} are malformed.");
                std::process::exit(-344);
            }
            Err(InteractError::HttpCode(404)) => {
                println!(
                    "Crate {id}, version {version}, and target {target} was {}! (Tar)",
                    "not found".if_supports_color(Stdout, |text| text.bright_red())
                );
                std::process::exit(-6);
            }
            Err(InteractError::HttpCode(code)) => {
                println!("Http error {code} for crate {id}.");
                std::process::exit(-7);
            }
            Err(_) => {
                println!("Connection error.");
                std::process::exit(-8);
            }
        };

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
        if !args.ci {
            println!(
                "{} reports... ",
                "Getting".if_supports_color(Stdout, |text| text.bright_blue())
            );

            let license_out = args.reports.contains(&REPORT_FLAGS[0].to_string());
            let license_dl = args.reports.contains(&REPORT_FLAGS[1].to_string());
            let deps_out = args.reports.contains(&REPORT_FLAGS[2].to_string());
            let deps_dl = args.reports.contains(&REPORT_FLAGS[3].to_string());
            let audit_out = args.reports.contains(&REPORT_FLAGS[4].to_string());
            let audit_dl = args.reports.contains(&REPORT_FLAGS[5].to_string());

            let mut report_path = cargo_bin.clone();
            report_path.push(format!(".prebuilt/reports/{id}/{version}"));
            let report_path = report_path;

            // license.report
            handle_report(
                &interact,
                &agent,
                &id,
                &version,
                &"license".to_string(),
                &report_path,
                license_out,
                license_dl,
            );
            // deps.report
            handle_report(
                &interact,
                &agent,
                &id,
                &version,
                &"deps".to_string(),
                &report_path,
                deps_out,
                deps_dl,
            );
            // audit.report
            handle_report(
                &interact,
                &agent,
                &id,
                &version,
                &"audit".to_string(),
                &report_path,
                audit_out,
                audit_dl,
            );
        }

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

fn handle_report(
    interact: &Box<dyn Interact>,
    agent: &Agent,
    id: &String,
    version: &String,
    name: &String,
    report_path: &Path,
    out: bool,
    dl: bool,
) {
    if out || dl {
        let report = match interact.get_report(agent, id, version, name) {
            Ok(r) => r,
            Err(InteractError::HttpCode(404)) => {
                println!("Could not find a {name} report in the index.");
                return;
            }
            Err(_) => {
                println!("Unknown error when trying to get {name} report.");
                return;
            }
        };

        if out {
            println!("{name}.report:\n{report}");
        }

        if dl {
            let mut dir = report_path.to_path_buf();
            match create_dir_all(&dir) {
                Ok(_) => {
                    dir.push(format!("{name}.report"));
                    match File::create(&dir) {
                        Ok(mut file) => match file.write(report.as_bytes()) {
                            Ok(_) => {}
                            Err(_) => {
                                println!("Could not write to {name}.report file.")
                            }
                        },
                        Err(_) => println!("Could not create {name}.report file."),
                    }
                }
                Err(_) => {
                    println!("Could not create directories for {name}.report.")
                }
            }
        }
    }
}
