use sha2::{Digest, Sha256};
use std::{
    env,
    fs::File,
    io::{Read, Write},
    path::Path,
    str,
};

static TARGET: &str = env!("TARGET");
static DOWNLOAD_URL: &str = "https://github.com/crow-rest/cargo-prebuilt-index/releases/download";

fn main() -> Result<(), String> {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    // Check if CARGO_HOME is set
    let cargo_home = env::var("CARGO_HOME").map_err(|_e| "$CARGO_HOME is not set.".to_string())?;
    let cargo_bin = format!("{}/bin", cargo_home);

    // Get pkg
    let pkgs = match args.get(0) {
        Some(args) => args.split(','),
        None => return Err("Missing pkgs in args.".to_string()),
    };

    for pkg in pkgs {
        let mut id = pkg;
        let mut version = None; // None will pull the latest version

        // If there is a version string get it
        if let Some((i, j)) = id.split_once('@') {
            id = i;
            version = Some(j)
        }

        // Download the index
        let res = reqwest::blocking::get(format!("{}/stable-index/{}", DOWNLOAD_URL, id)).map_err(
            |_e| "Error trying to get index, make sure your crate is listed.".to_string(),
        )?;
        let text = res.text().map_err(|_e| "Malformed response.".to_string())?;

        // Process index
        let text = text.trim();
        let lines: Vec<&str> = text.split('\n').collect();
        let lines = lines.iter().filter_map(|l| {
            if !l.starts_with('#') {
                Some(l.trim())
            }
            else {
                None
            }
        });

        let mut info = None;
        for l in lines {
            let (v, link) = match l.split_once(' ') {
                Some(ss) => ss,
                None => {
                    println!("Malformed index.");
                    std::process::exit(1);
                }
            };

            if version.is_none() || v.eq(version.unwrap()) {
                info = Some((v, link));
            }
        }

        if info.is_none() {
            return Err(format!("Version {:?} for {} not found.", version, id));
        }

        // Download the binary's index
        let res = reqwest::blocking::get(info.unwrap().1)
            .map_err(|_e| "Error trying to get binary index.".to_string())?;
        let text = res.text().map_err(|_e| "Malformed response.".to_string())?;

        // Process binary index
        let text = text.trim();
        let lines: Vec<&str> = text.split('\n').collect();

        let mut download = None;
        for l in lines {
            let (target, link) = match l.split_once(' ') {
                Some(ss) => ss,
                None => {
                    println!("Malformed binary index.");
                    std::process::exit(1);
                }
            };

            if target.eq(TARGET) {
                download = Some(link);
            }
        }

        if download.is_none() {
            return Err("Could not find a target triple that matches yours.".to_string());
        }

        let (d_zip, d_sha) = match download.unwrap().split_once(' ') {
            Some(ss) => ss,
            None => {
                println!("Malformed downloads.");
                std::process::exit(1);
            }
        };

        // Download zip
        let res =
            reqwest::blocking::get(d_zip).map_err(|_e| "Error trying to get zip.".to_string())?;
        let bytes = res
            .bytes()
            .map_err(|_e| "Could not get bytes from zip download.".to_string())?;

        // Download hash
        let res =
            reqwest::blocking::get(d_sha).map_err(|_e| "Error trying to get zip.".to_string())?;
        let d_sha = res
            .text()
            .map_err(|_e| "Malformed remote hash.".to_string())?;

        // Check hash
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash: Vec<u8> = hasher.finalize().to_vec();
        let hash = hex::encode(hash);

        if !(hash.eq(&d_sha)) {
            println!("Hashes do not match.");
            std::process::exit(256);
        }

        // Extract zip
        let reader = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(reader).unwrap();

        // Put bins
        let mut files = Vec::new();
        for f in archive.file_names() {
            if f.starts_with("bins/") {
                files.push(String::from(f));
            }
        }

        for f in files.iter_mut() {
            let mut bin = archive.by_name(f).unwrap();
            let _ = f.drain(0..5);

            let path_str = format!("{}/{}", cargo_bin, f);
            let path = Path::new(&path_str);
            let mut file =
                File::create(&path).map_err(|_e| "Could create file to write to.".to_string())?;

            let mut buffer = Vec::new();
            bin.read_to_end(&mut buffer).unwrap();
            file.write_all(&buffer).unwrap();

            // Try to allow execution on unix based systems
            #[cfg(target_family = "unix")]
            {
                use file_mode::ModePath;
                let _ = path.set_mode("+x");
            }

            println!("Installed {} at {}.", f, path_str);
        }
    }

    Ok(())
}
