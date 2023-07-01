use crate::{
    config::{Config, REPORT_FLAGS},
    data::{HashesFile, HashesFileImm, InfoFile, InfoFileImm},
    interact::{self, Interact, InteractError},
};
use owo_colors::{OwoColorize, Stream::Stderr};
use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
};
use ureq::Agent;

#[derive(Debug, Default)]
struct Data {
    id: Option<String>,
    version: Option<String>,
    info: Option<InfoFileImm>,
}

pub struct Fetcher {
    interact: Box<dyn Interact>,
    data: Data,
}
impl Fetcher {
    pub fn new(config: &Config, agent: Agent) -> Self {
        let interact = interact::create_interact(config.index.clone(), config.auth.as_ref(), agent);
        Self {
            interact,
            data: Default::default(),
        }
    }

    pub fn reset(&mut self) {
        self.data = Default::default();
    }

    pub fn get_version(&self) -> String {
        self.data
            .version
            .as_ref()
            .expect("Failed to get version, but it should have been guaranteed.")
            .to_string()
    }

    pub fn load(&mut self, id: &str, version: Option<&str>) {
        self.data.id = Some(id.to_string());
        self.data.version = match version {
            Some(v) => Some(v.to_string()),
            None => Some(self.fetch_latest()),
        };
    }

    pub fn download(&mut self, config: &Config) -> Vec<u8> {
        let id = self
            .data
            .id
            .as_ref()
            .expect("Failed to get id, but it should have been guaranteed.");
        let version = self
            .data
            .version
            .as_ref()
            .expect("Failed to get version, but it should have been guaranteed.");

        // info.json
        let raw_info_file = &self.fetch_str("info.json");
        let info: InfoFile = serde_json::from_str(raw_info_file)
            .expect(&format!("info.json is malformed for {id}@{version}"));
        let info: InfoFileImm = info.into();

        // info.json.sig and test
        #[cfg(feature = "sig")]
        if let Some(sig_file) = info.files.sig_info.clone() {
            self.verify_file("info.json", config, &sig_file, raw_info_file);
        }
        else if config.force_verify {
            eprintln!(
                "Could not force sig for index {}. info.json is not signed for {id}@{version}.",
                config.index
            );
            std::process::exit(224);
        }

        #[cfg(not(feature = "sig"))]
        if config.force_verify {
            eprintln!(
                "Could not force sig for index {}. Feature 'sig' is disabled.",
                config.index
            );
        }

        // check if target is supported
        if !info.targets.contains(&config.target) {
            eprintln!("{id}@{version} does not support target {}", config.target);
            std::process::exit(505);
        }

        // check if compression is supported
        if !info.archive.compression.eq("gz") {
            eprintln!("{id}@{version} does not support target {}", config.target);
            std::process::exit(505);
        }

        // hashes.json
        let raw_hashes_file = &self.fetch_str(&info.files.hash);
        let hashes: HashesFile = serde_json::from_str(raw_hashes_file).expect(&format!(
            "{} is malformed for {id}@{version}",
            info.files.hash
        ));
        let hashes: HashesFileImm = hashes.into();

        // hashes.json.sig and test
        #[cfg(feature = "sig")]
        if let Some(sig_file) = info.files.sig_hash.clone() {
            self.verify_file(&info.files.hash, config, &sig_file, raw_hashes_file);
        }
        else if config.force_verify {
            eprintln!(
                "Could not force sig for index {}. info.json is not signed for {id}@{version}.",
                config.index
            );
            std::process::exit(224);
        }

        // tar
        eprintln!(
            "{} {id}@{version} for target {}.",
            "Downloading".if_supports_color(Stderr, |text| text.bright_blue()),
            &config.target
        );
        let tar_bytes = self.fetch_blob(&format!("{}.{}", config.target, info.archive.ext));

        // test hashes
        self.verify_tar(config, &hashes, &tar_bytes);

        // store info for reports later if allowed
        self.data.info = Some(info);

        tar_bytes
    }

    pub fn reports(&mut self, config: &Config) {
        todo!();
    }

    fn fetch_latest(&self) -> String {
        let id = self
            .data
            .id
            .as_ref()
            .expect("Failed to get id, but it should have been guaranteed.");

        match self.interact.get_latest(id.as_str()) {
            Ok(s) => s,
            Err(InteractError::Malformed) => {
                eprintln!("The version string for {id} is malformed.");
                std::process::exit(342);
            }
            Err(InteractError::HttpCode(404)) => {
                eprintln!(
                    "Crate {id} {} in index!",
                    "not found".if_supports_color(Stderr, |text| text.bright_red())
                );
                std::process::exit(3);
            }
            Err(InteractError::HttpCode(code)) => {
                eprintln!("Http error {code} for crate {id}.");
                std::process::exit(4);
            }
            Err(err) => {
                eprintln!("Connection error.\n{err}");
                std::process::exit(5);
            }
        }
    }

    fn fetch_str(&self, file: &str) -> String {
        let id = self
            .data
            .id
            .as_ref()
            .expect("Failed to get id, but it should have been guaranteed.");
        let version = self
            .data
            .version
            .as_ref()
            .expect("Failed to get version, but it should have been guaranteed.");

        match self.interact.get_str(id, version, file) {
            Ok(s) => s,
            Err(InteractError::Malformed) => {
                eprintln!("The downloaded string {file} for {id}@{version} is malformed");
                std::process::exit(343);
            }
            Err(InteractError::HttpCode(404)) => {
                eprintln!(
                    "File {file} for {id}@{version} is {}!",
                    "not found".if_supports_color(Stderr, |text| text.bright_red())
                );
                std::process::exit(12);
            }
            Err(InteractError::HttpCode(code)) => {
                eprintln!("Http error {code} for {file} for {id}@{version}.");
                std::process::exit(13);
            }
            Err(err) => {
                eprintln!("Connection error.\n{err}");
                std::process::exit(14);
            }
        }
    }

    fn fetch_blob(&self, file: &str) -> Vec<u8> {
        let id = self
            .data
            .id
            .as_ref()
            .expect("Failed to get id, but it should have been guaranteed.");
        let version = self
            .data
            .version
            .as_ref()
            .expect("Failed to get version, but it should have been guaranteed.");

        match self.interact.get_blob(id, version, file) {
            Ok(s) => s,
            Err(InteractError::Malformed) => {
                eprintln!("The downloaded blob {file} for {id}@{version} is malformed");
                std::process::exit(343);
            }
            Err(InteractError::HttpCode(404)) => {
                eprintln!(
                    "File {file} for {id}@{version} is {}!",
                    "not found".if_supports_color(Stderr, |text| text.bright_red())
                );
                std::process::exit(24);
            }
            Err(InteractError::HttpCode(code)) => {
                eprintln!("Http error {code} for {file} for {id}@{version}.");
                std::process::exit(25);
            }
            Err(err) => {
                eprintln!("Connection error.\n{err}");
                std::process::exit(26);
            }
        }
    }

    #[cfg(feature = "sig")]
    fn verify_file(&self, file: &str, config: &Config, sig_file: &str, raw_file: &str) {
        use base64::Engine;
        use pgp::{Deserializable, SignedPublicKey, StandaloneSignature};

        let id = self
            .data
            .id
            .as_ref()
            .expect("Failed to get id, but it should have been guaranteed.");
        let version = self
            .data
            .version
            .as_ref()
            .expect("Failed to get version, but it should have been guaranteed.");

        let sig = &self.fetch_blob(sig_file);
        let mut reader = std::io::Cursor::new(sig);
        let sig = StandaloneSignature::from_bytes(&mut reader).unwrap();

        if config.sigs.contains_key(&config.index)
            && config.sigs.get(&config.index).unwrap().len() > 0
        {
            let mut verified = false;
            let keys = config.sigs.get(&config.index).unwrap();
            for key in keys {
                let raw_key = base64::engine::general_purpose::STANDARD
                    .decode(key)
                    .expect(&format!("A key for {} is malformed base64.", config.index));
                let mut reader = std::io::Cursor::new(raw_key);
                let pubkey = SignedPublicKey::from_armor_single(&mut reader).unwrap().0;

                match sig.verify(&pubkey, raw_file.as_bytes()) {
                    Ok(_) => verified = true,
                    Err(_) => {}
                }
            }

            if !verified {
                eprintln!("Could not verify {file} for {id}@{version}.");
                if config.force_verify {
                    std::process::exit(224);
                }
            }
            else {
                eprintln!(
                    "{} {file} for {id}@{version} with pgp.",
                    "Verified".if_supports_color(Stderr, |text| text.bright_blue())
                );
            }
        }
        else if config.force_verify {
            eprintln!(
                "Could not force sig for index {}. Missing public key.",
                config.index
            );
            std::process::exit(224);
        }
    }

    fn verify_tar(&self, config: &Config, hashes: &HashesFileImm, tar_bytes: &[u8]) {
        let id = self
            .data
            .id
            .as_ref()
            .expect("Failed to get id, but it should have been guaranteed.");
        let version = self
            .data
            .version
            .as_ref()
            .expect("Failed to get version, but it should have been guaranteed.");

        #[cfg(any(feature = "sha3", feature = "sha2"))]
        if let Some(blob) = hashes.hashes.get(&config.target) {
            let hashes = &blob.archive;

            #[cfg(feature = "sha3")]
            {
                use sha3::{Digest, Sha3_256, Sha3_512};

                // sha3_512
                if let Some(sha_hash) = hashes.get("sha3_512") {
                    let mut hasher = Sha3_512::new();
                    hasher.update(&tar_bytes);
                    let hash: Vec<u8> = hasher.finalize().to_vec();
                    let hash = hex::encode(hash);

                    if !(hash.eq(sha_hash)) {
                        eprintln!("Sha3_512 hashes do not match. {sha_hash} != {hash}");
                        std::process::exit(3512);
                    }

                    eprintln!(
                        "{} tar for {id}@{version} with Sha3_512.",
                        "Verified".if_supports_color(Stderr, |text| text.bright_blue())
                    );
                    return;
                }

                // sha3_256
                if let Some(sha_hash) = hashes.get("sha3_256") {
                    let mut hasher = Sha3_256::new();
                    hasher.update(&tar_bytes);
                    let hash: Vec<u8> = hasher.finalize().to_vec();
                    let hash = hex::encode(hash);

                    if !(hash.eq(sha_hash)) {
                        eprintln!("Sha3_256 hashes do not match. {sha_hash} != {hash}");
                        std::process::exit(3512);
                    }

                    eprintln!(
                        "{} tar for {id}@{version} with Sha3_256.",
                        "Verified".if_supports_color(Stderr, |text| text.bright_blue())
                    );
                    return;
                }
            }

            #[cfg(feature = "sha2")]
            {
                use sha2::{Digest, Sha256, Sha512};

                // sha512
                if let Some(sha_hash) = hashes.get("sha512") {
                    let mut hasher = Sha512::new();
                    hasher.update(&tar_bytes);
                    let hash: Vec<u8> = hasher.finalize().to_vec();
                    let hash = hex::encode(hash);

                    if !(hash.eq(sha_hash)) {
                        eprintln!("Sha512 hashes do not match. {sha_hash} != {hash}");
                        std::process::exit(512);
                    }

                    eprintln!(
                        "{} tar for {id}@{version} with Sha512.",
                        "Verified".if_supports_color(Stderr, |text| text.bright_blue())
                    );
                    return;
                }

                // sha256
                if let Some(sha_hash) = hashes.get("sha256") {
                    let mut hasher = Sha256::new();
                    hasher.update(&tar_bytes);
                    let hash: Vec<u8> = hasher.finalize().to_vec();
                    let hash = hex::encode(hash);

                    if !(hash.eq(sha_hash)) {
                        eprintln!("Sha256 hashes do not match. {sha_hash} != {hash}");
                        std::process::exit(256);
                    }

                    eprintln!(
                        "{} tar for {id}@{version} with Sha256.",
                        "Verified".if_supports_color(Stderr, |text| text.bright_blue())
                    );
                    return;
                }
            }
        }

        eprintln!("Could not verify downloaded tar file for {id}@{version}.");
        if config.force_verify {
            std::process::exit(228);
        }
    }
}

//pub fn latest_version(interact: &dyn Interact, id: &str) -> String {
//    match interact.get_latest(id) {
//        Ok(s) => s,
//        Err(InteractError::Malformed) => {
//            eprintln!("The version string for {id} is malformed.");
//            std::process::exit(342);
//        }
//        Err(InteractError::HttpCode(404)) => {
//            eprintln!(
//                "Crate {id} {} in index!",
//                "not found".if_supports_color(Stderr, |text| text.bright_red())
//            );
//            std::process::exit(3);
//        }
//        Err(InteractError::HttpCode(code)) => {
//            eprintln!("Http error {code} for crate {id}.");
//            std::process::exit(4);
//        }
//        Err(err) => {
//            eprintln!("Connection error.\n{err}");
//            std::process::exit(5);
//        }
//    }
//}

//pub fn hash(interact: &dyn Interact, id: &str, version: &str, target: &str) -> String {
//    match interact.get_hash(id, version, target) {
//        Ok(s) => s,
//        Err(InteractError::Malformed) => {
//            eprintln!("The hash string for {id} is malformed.");
//            std::process::exit(343);
//        }
//        Err(InteractError::HttpCode(404)) => {
//            eprintln!(
//                "Crate {id}, version {version}, and target {target} was {}! (Hash)",
//                "not found".if_supports_color(Stderr, |text| text.bright_red())
//            );
//            std::process::exit(9);
//        }
//        Err(InteractError::HttpCode(code)) => {
//            eprintln!("Http error {code} for crate {id}.");
//            std::process::exit(10);
//        }
//        Err(_) => {
//            eprintln!("Connection error.");
//            std::process::exit(11);
//        }
//    }
//}

//pub fn tar(interact: &dyn Interact, id: &str, version: &str, target: &str) -> Vec<u8> {
//    println!(
//        "{} {id} {version} from {}.tar.gz",
//        "Downloading".if_supports_color(Stderr, |text| text.bright_blue()),
//        interact.pre_url(id, version, target)
//    );
//
//    match interact.get_tar(id, version, target) {
//        Ok(b) => b,
//        Err(InteractError::Malformed) => {
//            eprintln!("The tar bytes for {id} are malformed.");
//            std::process::exit(344);
//        }
//        Err(InteractError::HttpCode(404)) => {
//            eprintln!(
//                "Crate {id}, version {version}, and target {target} was {}! (Tar)",
//                "not found".if_supports_color(Stderr, |text| text.bright_red())
//            );
//            std::process::exit(6);
//        }
//        Err(InteractError::HttpCode(code)) => {
//            eprintln!("Http error {code} for crate {id}.");
//            std::process::exit(7);
//        }
//        Err(_) => {
//            eprintln!("Connection error.");
//            std::process::exit(8);
//        }
//    }
//}
//
//pub fn reports(interact: &dyn Interact, args: &Config, path: &Path, id: &str, version: &str) {
//    if !args.ci {
//        println!(
//            "{} reports... ",
//            "Getting".if_supports_color(Stderr, |text| text.bright_blue())
//        );
//
//        let license_out = args.reports.contains(&REPORT_FLAGS[0].to_string());
//        let license_dl = args.reports.contains(&REPORT_FLAGS[1].to_string());
//        let deps_out = args.reports.contains(&REPORT_FLAGS[2].to_string());
//        let deps_dl = args.reports.contains(&REPORT_FLAGS[3].to_string());
//        let audit_out = args.reports.contains(&REPORT_FLAGS[4].to_string());
//        let audit_dl = args.reports.contains(&REPORT_FLAGS[5].to_string());
//
//        let mut report_path = path.to_path_buf();
//        report_path.push(format!(".prebuilt/reports/{id}/{version}"));
//        let report_path = report_path;
//
//        // license.report
//        handle_report(
//            interact,
//            id,
//            version,
//            "license",
//            &report_path,
//            license_out,
//            license_dl,
//        );
//        // deps.report
//        handle_report(
//            interact,
//            id,
//            version,
//            "deps",
//            &report_path,
//            deps_out,
//            deps_dl,
//        );
//        // audit.report
//        handle_report(
//            interact,
//            id,
//            version,
//            "audit",
//            &report_path,
//            audit_out,
//            audit_dl,
//        );
//    }
//}
//
//fn handle_report(
//    interact: &dyn Interact,
//    id: &str,
//    version: &str,
//    name: &str,
//    report_path: &Path,
//    out: bool,
//    dl: bool,
//) {
//    if out || dl {
//        let report = match interact.get_report(id, version, name) {
//            Ok(r) => r,
//            Err(InteractError::HttpCode(404)) => {
//                eprintln!("Could not find a {name} report in the index.");
//                return;
//            }
//            Err(_) => {
//                eprintln!("Unknown error when trying to get {name} report.");
//                return;
//            }
//        };
//
//        if out {
//            println!("{name}.report:\n{report}");
//        }
//
//        if dl {
//            let mut dir = report_path.to_path_buf();
//            match create_dir_all(&dir) {
//                Ok(_) => {
//                    dir.push(format!("{name}.report"));
//                    match File::create(&dir) {
//                        Ok(mut file) => match file.write(report.as_bytes()) {
//                            Ok(_) => {}
//                            Err(_) => {
//                                eprintln!("Could not write to {name}.report file.")
//                            }
//                        },
//                        Err(_) => eprintln!("Could not create {name}.report file."),
//                    }
//                }
//                Err(_) => {
//                    eprintln!("Could not create directories for {name}.report.")
//                }
//            }
//        }
//    }
//}
