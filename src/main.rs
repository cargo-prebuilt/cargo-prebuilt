#[cfg(test)]
mod test;

use flate2::read::GzDecoder;
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
static DOWNLOAD_URL: &str = "https://github.com/crow-rest/cargo-prebuilt-index/releases/download";
static REPORT_FLAGS: [&str; 6] = [
    "license-out",
    "license-dl",
    "deps-out",
    "deps-dl",
    "audit-out",
    "audit-dl",
];

fn main() -> Result<(), String> {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    // Remove prebuilt if running by "cargo prebuilt"
    match args.get(0) {
        None => {
            println!("Error not enough args. Try --help.");
        }
        Some(a) => {
            if a.eq("prebuilt") {
                args.remove(0);
            }
        }
    }

    // Process args
    let mut pkgs = None;
    let mut target = TARGET.to_string();
    let mut no_bin = false;
    let mut ci = false;
    let mut no_create_ch = false;
    let mut reports = vec!["license-dl".to_string()];
    for mut arg in args {
        if arg.starts_with("--") {
            if arg.starts_with("--target=") {
                arg.replace_range(0..9, "");
                target = arg;
            }
            else if arg.eq("--no-bin") {
                no_bin = true;
            }
            else if arg.eq("--ci") {
                ci = true;
            }
            else if arg.eq("--no-create") {
                no_create_ch = true;
            }
            else if arg.starts_with("--reports=") {
                arg.replace_range(0..10, "");
                reports = arg
                    .split(',')
                    .map(|i| {
                        if !REPORT_FLAGS.contains(&i) {
                            println!("Not a valid report flag: {i}");
                            std::process::exit(-33);
                        }
                        i.to_string()
                    })
                    .collect()
            }
            else if arg.eq("--nightly") {
                println!("--nightly is not implemented yet.");
                std::process::exit(-1);
            }
            else if arg.eq("--version") {
                println!(env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            else if arg.eq("--help") {
                println!("See https://github.com/crow-rest/cargo-prebuilt#how-to-use");
                std::process::exit(0);
            }
        }
        else {
            pkgs = Some(arg);
        }
    }
    let pkgs = pkgs;
    let target = target.as_str();
    let no_bin = no_bin;

    // Get location to install binaries to
    let mut cargo_home = detect_cargo();
    if !no_bin && !cargo_home.ends_with("bin") {
        cargo_home.push("bin");
    }
    let cargo_bin = cargo_home;
    if !no_create_ch && create_dir_all(&cargo_bin).is_err() {
        println!("Could not create the dirs {cargo_bin:?}.");
        std::process::exit(-44);
    }

    let agent = ureq::AgentBuilder::new()
        .tls_connector(Arc::new(
            native_tls::TlsConnector::new().expect("Could not create TlsConnector"),
        ))
        .https_only(true)
        .build();

    // Get pkgs
    let pkgs: Vec<String> = match pkgs {
        Some(args) => args.split(',').map(|s| s.to_string()).collect(),
        None => {
            println!("Missing pkgs in args.");
            std::process::exit(-2);
        }
    };

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
            let res = match agent
                .get(&format!("{DOWNLOAD_URL}/stable-index/{id}"))
                .call()
            {
                Ok(response) => {
                    let s = response
                        .into_string()
                        .expect("Malformed latest version string.");
                    s.trim().to_string()
                }
                Err(Error::Status(code, _)) => {
                    if code == 404 {
                        println!("Crate {id} not found in index!");
                        std::process::exit(-3);
                    }
                    else {
                        println!("Error {code} for crate {id}.");
                        std::process::exit(-4);
                    }
                }
                Err(_) => {
                    println!("Connection error.");
                    std::process::exit(-5);
                }
            };

            version = Some(res);
        }

        let version = version.unwrap();

        // Download package
        let base_url = format!("{DOWNLOAD_URL}/{id}-{version}/");
        let pre_url = format!("{base_url}{target}");

        let sha_hash = match agent.get(&format!("{pre_url}.sha256")).call() {
            Ok(response) => {
                let s = response.into_string().expect("Malformed hash string.");
                s.trim().to_string()
            }
            Err(Error::Status(code, _)) => {
                if code == 404 {
                    println!(
                        "Crate {id}, version {version}, and target {target} was not found! (Hash)",
                    );
                    std::process::exit(-9);
                }
                else {
                    println!("Error {code} for crate {id}.");
                    std::process::exit(-10);
                }
            }
            Err(_) => {
                println!("Connection error.");
                std::process::exit(-11);
            }
        };

        let mut tar_bytes: Vec<u8> = Vec::new();
        println!("Downloading {id} {version} from {pre_url}.tar.gz");
        match agent.get(&format!("{pre_url}.tar.gz")).call() {
            Ok(response) => {
                response
                    .into_reader()
                    .read_to_end(&mut tar_bytes)
                    .expect("Failed when reading in tar.gz bytes.");
            }
            Err(Error::Status(code, _)) => {
                if code == 404 {
                    println!("Crate {id}, version {version}, and target {target} was not found!");
                    std::process::exit(-6);
                }
                else {
                    println!("Error {code} for crate {id}.");
                    std::process::exit(-7);
                }
            }
            Err(_) => {
                println!("Connection error.");
                std::process::exit(-8);
            }
        }

        // Check hash
        let mut hasher = Sha256::new();
        hasher.update(&tar_bytes);
        let hash: Vec<u8> = hasher.finalize().to_vec();
        let hash = hex::encode(hash);

        if !(hash.eq(&sha_hash)) {
            println!("Hashes do not match.");
            std::process::exit(-256);
        }

        // Untar Tar
        let reader = std::io::Cursor::new(tar_bytes);
        let mut archive = Archive::new(GzDecoder::new(reader));
        match archive.entries() {
            Ok(es) => {
                println!("Extracting {id} {version}...");

                for e in es {
                    let mut e = e.expect("Malformed entry.");

                    let mut path = cargo_bin.clone();
                    path.push(e.path().expect("Could not extract path from tar."));

                    e.unpack(&path)
                        .expect("Could not extract binaries from downloaded tar archive");

                    let abs = fs::canonicalize(path).expect("Could not canonicalize install path");
                    println!("Added {abs:?}.");
                }
            }
            Err(_) => {
                println!("Connection error.");
                std::process::exit(-13);
            }
        }

        // Reports
        if !ci {
            println!("Getting reports... ");

            let license_out = reports.contains(&REPORT_FLAGS[0].to_string());
            let license_dl = true; // On by default.
            let deps_out = reports.contains(&REPORT_FLAGS[2].to_string());
            let deps_dl = reports.contains(&REPORT_FLAGS[3].to_string());
            let audit_out = reports.contains(&REPORT_FLAGS[4].to_string());
            let audit_dl = reports.contains(&REPORT_FLAGS[5].to_string());

            let mut report_path = cargo_bin.clone();
            report_path.push(format!(".prebuilt/reports/{id}/{version}"));
            let report_path = report_path;

            // license.report
            handle_report(
                &agent,
                "license",
                &base_url,
                &report_path,
                license_out,
                license_dl,
            );
            // deps.report
            handle_report(&agent, "deps", &base_url, &report_path, deps_out, deps_dl);
            // audit.report
            handle_report(
                &agent,
                "audit",
                &base_url,
                &report_path,
                audit_out,
                audit_dl,
            );

            println!("Done getting reports.");
        }

        println!("Installed {id} {version}.");
    }

    println!("Done!");

    Ok(())
}

fn detect_cargo() -> PathBuf {
    // Use CARGO_HOME env var.
    if let Some(home) = env::var_os("CARGO_HOME") {
        return PathBuf::from(home);
    }

    // Try to find CARGO_HOME by searching for cargo executable in common paths.
    let ext = if TARGET.contains("windows") { ".exe" } else { "" };
    let mut home_cargo_dir = home::home_dir().unwrap_or_else(|| PathBuf::from("~"));
    home_cargo_dir.push(".cargo/bin/cargo");
    for path in [home_cargo_dir, PathBuf::from(format!("/usr/local/cargo/bin/cargo{ext}"))].iter_mut() {
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
    }

    println!("Platform does not support which/where.exe detection of cargo. Please set the CARGO_HOME env var.");
    std::process::exit(-122);
}

fn handle_report(
    agent: &Agent,
    name: &str,
    base_url: &String,
    report_path: &Path,
    out: bool,
    dl: bool,
) {
    if out || dl {
        let url = format!("{base_url}{name}.report");
        match agent.get(&url).call() {
            Ok(response) => {
                let mut bytes: Vec<u8> = Vec::new();
                if response.into_reader().read_to_end(&mut bytes).is_ok() {
                    if let Ok(s) = String::from_utf8(bytes) {
                        if out {
                            println!("{name}.report:\n{s}");
                        }

                        if dl {
                            let mut dir = report_path.to_path_buf();
                            match create_dir_all(&dir) {
                                Ok(_) => {
                                    dir.push(format!("{name}.report"));
                                    match File::create(&dir) {
                                        Ok(mut file) => match file.write(s.as_bytes()) {
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
            }
            Err(Error::Status(code, _)) => {
                if code == 404 {
                    println!("Did not find a {name} report in the index.");
                }
            }
            Err(_) => {
                println!("Connection error.");
            }
        }
    }
}
