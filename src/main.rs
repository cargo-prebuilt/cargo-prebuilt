use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use std::{env, io::Read, path::PathBuf, str, sync::Arc};
use tar::Archive;
use ureq::Error;

static TARGET: &str = env!("TARGET");
static DOWNLOAD_URL: &str = "https://github.com/crow-rest/cargo-prebuilt-index/releases/download";

fn main() -> Result<(), String> {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    if args.len() > 1 {
        args.remove(0);
    }

    // Check if CARGO_HOME is set
    let cargo_home = env::var("CARGO_HOME").map_err(|_e| "$CARGO_HOME is not set.".to_string())?;
    let cargo_bin = format!("{}/bin", cargo_home);

    let agent = ureq::AgentBuilder::new()
        .tls_connector(Arc::new(
            native_tls::TlsConnector::new().expect("Could not create TlsConnector"),
        ))
        .build();

    // Get pkgs
    let pkgs = match args.get(0) {
        Some(args) => args.split(','),
        None => return Err("Missing pkgs in args.".to_string()),
    };

    for pkg in pkgs {
        let mut id = pkg;
        let mut version: Option<String> = None; // None will pull the latest version

        // If there is a version string get it
        if let Some((i, j)) = id.split_once('@') {
            id = i;
            version = Some(j.to_string())
        }

        // Get latest version
        if version.is_none() {
            let res = match agent
                .get(&format!("{}/stable-index/{}", DOWNLOAD_URL, id))
                .call()
            {
                Ok(response) => {
                    let s = response.into_string().expect("Malformed latest string.");
                    s.trim().to_string()
                }
                Err(Error::Status(code, _)) => {
                    if code == 404 {
                        panic!("Crate {} not found in index!", id);
                    }
                    else {
                        panic!("Error {} for crate {}. (1)", code, id);
                    }
                }
                Err(_) => panic!("Connection error."),
            };

            version = Some(res);
        }

        let version = version.unwrap();

        // Download package
        let pre_url = format!("{}/{}-{}/{}", DOWNLOAD_URL, id, version, TARGET);

        let mut tar_bytes: Vec<u8> = Vec::new();
        match agent.get(&format!("{}.tar.gz", pre_url)).call() {
            Ok(response) => {
                response
                    .into_reader()
                    .read_to_end(&mut tar_bytes)
                    .expect("Failed when reading in tar.gz bytes.");
            }
            Err(Error::Status(code, _)) => {
                if code == 404 {
                    panic!(
                        "Crate {}, version {}, and target {} was not found!",
                        id, version, TARGET
                    );
                }
                else {
                    panic!("Error {} for crate {}. (2)", code, id);
                }
            }
            Err(_) => panic!("Connection error."),
        }

        let sha_hash = match agent.get(&format!("{}.sha256", pre_url)).call() {
            Ok(response) => {
                let s = response.into_string().expect("Malformed hash string.");
                s.trim().to_string()
            }
            Err(Error::Status(code, _)) => {
                if code == 404 {
                    panic!(
                        "Crate {}, version {}, and target {} was not found! (Hash)",
                        id, version, TARGET
                    );
                }
                else {
                    panic!("Error {} for crate {}. (3)", code, id);
                }
            }
            Err(_) => panic!("Connection error."),
        };

        // Check hash
        let mut hasher = Sha256::new();
        hasher.update(&tar_bytes);
        let hash: Vec<u8> = hasher.finalize().to_vec();
        let hash = hex::encode(hash);

        if !(hash.eq(&sha_hash)) {
            println!("Hashes do not match.");
            std::process::exit(256);
        }

        // Untar Tar
        let reader = std::io::Cursor::new(tar_bytes);
        let mut archive = Archive::new(GzDecoder::new(reader));
        match archive.entries() {
            Ok(es) => {
                for e in es {
                    let mut e = e.expect("Malformed entry.");
                    let path = PathBuf::from(format!(
                        "{}/{}",
                        cargo_bin,
                        e.path().unwrap().to_str().unwrap()
                    ));
                    e.unpack(path).expect("Could not extract bins from tar");
                }
            }
            Err(_) => panic!("Downloaded tar failed to be read."),
        }
    }

    println!("Done!");

    Ok(())
}
