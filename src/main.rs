use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use std::{env, fs::File, io::Read, path::PathBuf, str, sync::Arc};
use tar::Archive;
use ureq::Error;

static TARGET: &str = env!("TARGET");
static DOWNLOAD_URL: &str = "https://github.com/crow-rest/cargo-prebuilt-index/releases/download";
static REPORT_FLAGS: [&str; 6] = ["license-out", "license-dl", "deps-out", "deps-dl", "audit-out", "audit-dl"];

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
    let mut reports = vec!["license-dl"];
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
            else if arg.starts_with("--reports=") {
                arg.replace_range(0..10, "");
                reports = arg.split(",").map(|i| {
                    if !REPORT_FLAGS.contains(&i) {
                        println!("Not a valid report flag: {i}");
                        std::process::exit(-33);
                    }
                    i
                }).collect()
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
    let mut cargo_home = PathBuf::from(env::var_os("CARGO_HOME").unwrap_or_else(|| {
        let ext = if TARGET.contains("windows") {
            ".exe"
        }
        else {
            ""
        };
        match File::open(format!("~/.cargo/bin/cargo{ext}")) {
            Ok(_) => {
                println!("Detected cargo in ~/.cargo/bin/. Will install here.");
                "~/.cargo".into()
            }
            Err(_) => match File::open(format!("/usr/local/cargo/bin/cargo{ext}")) {
                Ok(_) => {
                    println!("Detected cargo in /usr/local/cargo/bin/. Will install here.");
                    "/usr/local/cargo".into()
                }
                Err(_) => {
                    println!("Could not detect cargo, please set the CARGO_HOME env variable.");
                    std::process::exit(-22);
                }
            },
        }
    }));
    if !no_bin {
        cargo_home.push("bin");
    }
    let cargo_bin = cargo_home;

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
        let pre_url = format!("{DOWNLOAD_URL}/{id}-{version}/{target}");

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
                    println!("Added {path:?}.");
                }

                println!("Installed {id} {version}.");
            }
            Err(_) => {
                println!("Connection error.");
                std::process::exit(-13);
            }
        }
    }

    println!("Done!");

    Ok(())
}
