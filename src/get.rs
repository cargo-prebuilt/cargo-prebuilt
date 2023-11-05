use std::{
    fs::{create_dir_all, File},
    io::Write,
};

use crate::{
    color::{err_color_print, PossibleColor},
    config::Config,
    data::{HashType, Hashes, HashesFile, HashesFileImm, InfoFile, InfoFileImm, ReportType},
    events,
    interact::{self, Interact, InteractError},
};
use ureq::Agent;

pub struct Fetcher {
    interact: Box<dyn Interact>,
}
impl Fetcher {
    pub fn new(config: &Config, agent: Agent) -> Self {
        let interact = interact::create_interact(config.index.clone(), config.auth.as_ref(), agent);
        Self { interact }
    }

    pub fn get_latest(&mut self, id: &str) -> String {
        self.fetch_latest(id)
    }

    pub fn download(
        &mut self,
        id: &str,
        version: &str,
        config: &Config,
    ) -> (InfoFileImm, HashesFileImm, Vec<u8>) {
        eprintln!(
            "{} info for {id}@{version}.",
            err_color_print("Fetching", PossibleColor::BrightBlue),
        );

        // info.json
        let raw_info_file = &self.fetch_str(id, version, "info.json");
        let info: InfoFile = serde_json::from_str(raw_info_file)
            .unwrap_or_else(|_| panic!("info.json is malformed for {id}@{version}"));
        let info: InfoFileImm = info.into();

        // info.json.minisig and test
        #[cfg(feature = "sig")]
        if !config.no_verify {
            if let Some(sig_file) = info.files.sig_info.clone() {
                let v =
                    self.verify_file(id, version, "info.json", config, &sig_file, raw_info_file);
                events::info_verify(id, version, config, v);
            }
            else {
                panic!(
                    "Could not force sig for index {}. info.json is not signed for {id}@{version}.",
                    config.index
                );
            }
        }
        #[cfg(not(feature = "sig"))]
        if !config.no_verify {
            panic!("Could not force sig for index {}. This requires the 'security' and/or 'sig' feature(s). Or you can use the flag '--no-verify'.", config.index);
        }

        // check if target is supported
        if !info.targets.contains(&config.target) {
            panic!(
                "{id}@{version} does {} target {}",
                err_color_print("not support", PossibleColor::BrightRed),
                config.target
            );
        }

        // check if compression is supported
        if !info.archive.compression.eq("gz") {
            panic!("{id}@{version} does not support compression gzip");
        }

        // check if binary does not exist, if safe mode is on
        if config.safe && !config.ci {
            for bin in info.bins.iter() {
                let mut path = config.path.clone();
                path.push(bin);

                if path.exists() {
                    panic!(
                        "Binary {bin} {} for {id}@{version}",
                        err_color_print("already exists", PossibleColor::BrightRed)
                    );
                }
            }
        }

        eprintln!(
            "{} hashes for {id}@{version} with target {}.",
            err_color_print("Fetching", PossibleColor::BrightBlue),
            &config.target
        );

        // hashes.json
        let raw_hashes_file = &self.fetch_str(id, version, &info.files.hash);
        let hashes: HashesFile = serde_json::from_str(raw_hashes_file)
            .unwrap_or_else(|_| panic!("{} is malformed for {id}@{version}", info.files.hash));
        let hashes: HashesFileImm = hashes.into();

        // hashes.json.minisig and test
        #[cfg(feature = "sig")]
        if !config.no_verify {
            if let Some(sig_file) = info.files.sig_hash.clone() {
                let v = self.verify_file(
                    id,
                    version,
                    &info.files.hash,
                    config,
                    &sig_file,
                    raw_hashes_file,
                );
                events::hashes_verify(id, version, config, v);
            }
            else {
                panic!(
                    "Could not force sig for index {}. hashes.json is not signed for {id}@{version}.",
                    config.index
                );
            }
        }

        // tar
        eprintln!(
            "{} {id}@{version} for target {}.",
            err_color_print("Downloading", PossibleColor::BrightYellow),
            &config.target
        );
        let tar_bytes = self.fetch_blob(
            id,
            version,
            &format!("{}.{}", config.target, info.archive.ext),
        );

        // test hashes
        self.verify_archive(id, version, config, &hashes, &tar_bytes);

        (info, hashes, tar_bytes)
    }

    pub fn is_bin(&self, info: &InfoFileImm, bin_name: &str) -> bool {
        let bin_name = bin_name.replace(".exe", "");
        info.bins.contains(&bin_name)
    }

    pub fn reports(&mut self, id: &str, version: &str, info: &InfoFileImm, config: &Config) {
        if config.reports.is_empty() {
            return;
        }

        eprintln!(
            "{} reports... ",
            err_color_print("Getting", PossibleColor::BrightBlue)
        );

        for report in config.reports.iter() {
            let report_name = match report {
                ReportType::LicenseDL => info.files.license.clone(),
                ReportType::DepsDL => info.files.deps.clone(),
                ReportType::AuditDL => info.files.audit.clone(),
            };

            let raw_str = &self.fetch_str(id, version, &report_name);

            let mut dir = config.report_path.clone();
            dir.push(format!("{id}/{version}"));
            match create_dir_all(&dir) {
                Ok(_) => {
                    dir.push(&report_name);
                    match File::create(&dir) {
                        Ok(mut file) => match file.write(raw_str.as_bytes()) {
                            Ok(_) => {}
                            Err(_) => {
                                eprintln!("Could not write to {report_name} file.")
                            }
                        },
                        Err(_) => eprintln!("Could not create {report_name} file."),
                    }
                }
                Err(_) => {
                    eprintln!("Could not create directories for {report_name}.")
                }
            }
        }
    }

    fn fetch_latest(&mut self, id: &str) -> String {
        match self.interact.get_latest(id) {
            Ok(s) => s,
            Err(InteractError::Malformed) => panic!("The version string for {id} is malformed."),
            Err(InteractError::HttpCode(404)) => panic!(
                "Crate {id} {} in index!",
                err_color_print("not found", PossibleColor::BrightRed)
            ),
            Err(InteractError::HttpCode(code)) => panic!("Http error {code} for crate {id}."),
            Err(err) => panic!("Connection error.\n{err}"),
        }
    }

    fn fetch_str(&mut self, id: &str, version: &str, file: &str) -> String {
        match self.interact.get_str(id, version, file) {
            Ok(s) => s,
            Err(InteractError::Malformed) => {
                panic!("The downloaded string {file} for {id}@{version} is malformed")
            }
            Err(InteractError::HttpCode(404)) => panic!(
                "File {file} for {id}@{version} is {}!",
                err_color_print("not found", PossibleColor::BrightRed)
            ),
            Err(InteractError::HttpCode(code)) => {
                panic!("Http error {code} for {file} for {id}@{version}.")
            }
            Err(err) => panic!("Connection error.\n{err}"),
        }
    }

    fn fetch_blob(&mut self, id: &str, version: &str, file: &str) -> Vec<u8> {
        match self.interact.get_blob(id, version, file) {
            Ok(s) => s,
            Err(InteractError::Malformed) => {
                panic!("The downloaded blob {file} for {id}@{version} is malformed")
            }
            Err(InteractError::HttpCode(404)) => panic!(
                "File {file} for {id}@{version} is {}!",
                err_color_print("not found", PossibleColor::BrightRed)
            ),
            Err(InteractError::HttpCode(code)) => {
                panic!("Http error {code} for {file} for {id}@{version}.")
            }
            Err(err) => panic!("Connection error.\n{err}"),
        }
    }

    #[cfg(feature = "sig")]
    fn verify_file(
        &mut self,
        id: &str,
        version: &str,
        file: &str,
        config: &Config,
        sig_file: &str,
        raw_file: &str,
    ) -> bool {
        use minisign_verify::{PublicKey, Signature};

        let sig = &self.fetch_str(id, version, sig_file);
        let signature = Signature::decode(sig).expect("Signature was malformed.");

        let mut verified = false;
        for key in config.sigs.iter() {
            let pk = PublicKey::from_base64(key).expect("Public key was malformed.");
            if pk.verify(raw_file.as_bytes(), &signature, false).is_ok() {
                verified = true;
                break;
            }
        }

        if !verified {
            panic!(
                "{} verify {file} for {id}@{version}.",
                err_color_print("Could not", PossibleColor::BrightRed)
            );
        }
        else {
            eprintln!(
                "{} {file} for {id}@{version} with minisign.",
                err_color_print("Verified", PossibleColor::BrightWhite)
            );
        }

        verified
    }

    fn verify_archive(
        &self,
        id: &str,
        version: &str,
        config: &Config,
        hashes: &HashesFileImm,
        bytes: &[u8],
    ) {
        if let Some(blob) = hashes.hashes.get(&config.target) {
            let hashes = &blob.archive;
            self.verify_bytes(
                id,
                version,
                hashes,
                &format!("{} archive", &config.target),
                bytes,
            )
        }
    }

    fn verify_bytes(&self, id: &str, version: &str, hashes: &Hashes, item: &str, bytes: &[u8]) {
        #[cfg(feature = "sha3")]
        {
            use sha3::{Digest, Sha3_256, Sha3_512};

            // sha3_512
            if let Some(sha_hash) = hashes.get(&HashType::Sha3_512) {
                let mut hasher = Sha3_512::new();
                hasher.update(bytes);
                let hash: Vec<u8> = hasher.finalize().to_vec();
                let hash = hex::encode(hash);

                if !(hash.eq(sha_hash)) {
                    panic!("sha3_512 hashes do not match for {item}. {sha_hash} != {hash}");
                }

                eprintln!(
                    "{} {item} for {id}@{version} with sha3_512.",
                    err_color_print("Verified", PossibleColor::BrightWhite)
                );
                return;
            }

            // sha3_256
            if let Some(sha_hash) = hashes.get(&HashType::Sha3_256) {
                let mut hasher = Sha3_256::new();
                hasher.update(bytes);
                let hash: Vec<u8> = hasher.finalize().to_vec();
                let hash = hex::encode(hash);

                if !(hash.eq(sha_hash)) {
                    panic!("sha3_256 hashes do not match for {item}. {sha_hash} != {hash}");
                }

                eprintln!(
                    "{} {item} for {id}@{version} with sha3_256.",
                    err_color_print("Verified", PossibleColor::BrightWhite)
                );
                return;
            }
        }

        #[cfg(feature = "sha2")]
        {
            use sha2::{Digest, Sha256, Sha512};

            // sha512
            if let Some(sha_hash) = hashes.get(&HashType::Sha512) {
                let mut hasher = Sha512::new();
                hasher.update(bytes);
                let hash: Vec<u8> = hasher.finalize().to_vec();
                let hash = hex::encode(hash);

                if !(hash.eq(sha_hash)) {
                    panic!("sha512 hashes do not match for {item}. {sha_hash} != {hash}");
                }

                eprintln!(
                    "{} {item} for {id}@{version} with sha512.",
                    err_color_print("Verified", PossibleColor::BrightWhite)
                );
                return;
            }

            // sha256
            if let Some(sha_hash) = hashes.get(&HashType::Sha256) {
                let mut hasher = Sha256::new();
                hasher.update(bytes);
                let hash: Vec<u8> = hasher.finalize().to_vec();
                let hash = hex::encode(hash);

                if !(hash.eq(sha_hash)) {
                    panic!("sha256 hashes do not match for {item}. {sha_hash} != {hash}");
                }

                eprintln!(
                    "{} {item} for {id}@{version} with sha256.",
                    err_color_print("Verified", PossibleColor::BrightWhite)
                );
                return;
            }
        }

        #[cfg(not(any(feature = "sha2", feature = "sha3")))]
        eprintln!("Could not verify downloaded {item} for {id}@{version}. This requires the 'security', 'sha3', and/or 'sha2' feature(s).");

        #[cfg(any(feature = "sha2", feature = "sha3"))]
        panic!("Could not verify downloaded {item} for {id}@{version}.");
    }

    // TODO: Use for update hashing.
    // No need to verify bins, since archive hash should do this for us.
    //    pub fn verify_bin(&self, config: &Config, bin_name: &str, bytes: &[u8]) {
    //        let id = self
    //            .data
    //            .id
    //            .as_ref()
    //            .expect("Failed to get id, but it should have been guaranteed.");
    //        let version = self
    //            .data
    //            .version
    //            .as_ref()
    //            .expect("Failed to get version, but it should have been guaranteed.");
    //        let hashes = self
    //            .data
    //            .hashes
    //            .as_ref()
    //            .expect("Failed to get hashes, but it should have been guaranteed.");
    //
    //        if let Some(blob) = hashes.hashes.get(&config.target) {
    //            match &blob.bins.get(bin_name) {
    //                Some(blob) => self.verify_bytes(blob, &format!("{bin_name} binary"), bytes),
    //                None => panic!("Could not find {bin_name} hash for {id}@{version}."),
    //            }
    //        }
    //    }
}
