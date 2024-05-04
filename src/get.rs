use std::{
    fs::{create_dir_all, File},
    io::Write,
};

use crate::{
    color,
    config::Config,
    data::{HashType, Hashes, HashesFile, HashesFileV1, InfoFile, InfoFileImm, ReportType},
    events,
    interact::{self, Interact, InteractError},
};
use ureq::Agent;

pub struct Fetcher {
    interact: Box<dyn Interact>,
}
impl Fetcher {
    pub fn new(config: &Config, agent: Agent) -> Self {
        let interact = interact::create_interactive(&config.index, config.auth.as_ref(), agent);
        Self { interact }
    }

    pub fn get_latest(&mut self, id: &str) -> String {
        self.fetch_latest(id)
    }

    pub fn download(&mut self, id: &str, version: &str, config: &Config) -> (InfoFileImm, Vec<u8>) {
        eprintln!(
            "{} info for {id}@{version}.",
            color!(bright_blue, "Fetching"),
        );

        // info.json
        let raw_info_file = &self.fetch_str(id, version, "info.json");

        // info.json verify
        if !config.no_sig {
            let v = self.verify_file(
                id,
                version,
                "info.json",
                config,
                "info.json.minisig",
                raw_info_file,
            );
            events::info_verify(id, version, config, v);
        }

        let info: InfoFile = serde_json::from_str(raw_info_file)
            .unwrap_or_else(|_| panic!("info.json is malformed for {id}@{version}"));
        let mut info: InfoFileImm = InfoFileImm::convert(info, &config.target);

        assert!(
            info.id.eq(id),
            "{id}@{version} does not match with info.json id {}",
            info.id
        );
        assert!(
            info.version.eq(version),
            "{id}@{version} does not match with info.json version {}",
            info.version
        );

        // check if compression is supported
        assert!(
            info.archive.compression.eq("gz"),
            "{id}@{version} does not support compression gzip"
        );

        // check if binary does not exist if safe mode is on
        if config.safe && !config.ci {
            for bin in &info.bins {
                let mut path = config.path.clone();
                path.push(bin);

                assert!(
                    !path.exists(),
                    "Binary {bin} {} for {id}@{version}",
                    color!(bright_red, "already exists")
                );
            }
        }

        if !config.no_hash {
            if let Some(ref polyfill) = info.polyfill {
                eprintln!(
                    "{} hashes for {id}@{version} with target {}.",
                    color!(bright_blue, "Fetching"),
                    &config.target
                );

                // hashes.json
                let raw_hashes_file = &self.fetch_str(id, version, &polyfill.hash_file);

                // hashes.json.minisig and test
                if !config.no_sig {
                    if let Some(sig_file) = polyfill.hash_file_sig.clone() {
                        let v = self.verify_file(
                            id,
                            version,
                            &polyfill.hash_file,
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

                let hashes: HashesFile =
                    serde_json::from_str(raw_hashes_file).unwrap_or_else(|_| {
                        panic!("{} is malformed for {id}@{version}", polyfill.hash_file)
                    });
                let hashes: HashesFileV1 = hashes.into();
                let hashes = hashes
                    .hashes
                    .get(&config.target)
                    .unwrap_or_else(|| panic!("No hashes for target {}", config.target));

                info.archive_hashes.clone_from(&hashes.archive);
                info.bins_hashes.clone_from(&hashes.bins);
            }
        }

        // check if target is supported, based on hash
        if !config.no_hash {
            assert!(
                !info.archive_hashes.is_empty(),
                "{id}@{version} does {} target {}, due to empty archive hashes",
                color!(bright_red, "not support"),
                config.target
            );
        }

        // TODO: For updating, tar downloading and verifying needs to be moved to another location.

        // tar
        eprintln!(
            "{} {id}@{version} for target {}.",
            color!(bright_yellow, "Downloading"),
            &config.target
        );
        let tar_bytes = self.fetch_blob(id, version, &info.archive_name);

        // test hashes
        Self::verify_archive(id, version, config, &info, &tar_bytes);

        (info, tar_bytes)
    }

    pub fn is_bin(info: &InfoFileImm, bin_name: &str) -> bool {
        let bin_name = bin_name.replace(".exe", "");
        info.bins.contains(&bin_name)
    }

    pub fn reports(&mut self, id: &str, version: &str, info: &InfoFileImm, config: &Config) {
        if config.reports.is_empty() {
            return;
        }

        eprintln!("{} reports... ", color!(bright_blue, "Getting"));

        for report in &config.reports {
            let report_name = match report {
                ReportType::LicenseDL | ReportType::LicenseEvent => info.files.license.clone(),
                ReportType::DepsDL | ReportType::DepsEvent => info.files.deps.clone(),
                ReportType::AuditDL | ReportType::AuditEvent => info.files.audit.clone(),
                ReportType::InfoJsonDL | ReportType::InfoJsonEvent => "info.json".to_string(),
            };

            let raw_str = &self.fetch_str(id, version, &report_name);

            match report {
                ReportType::LicenseDL
                | ReportType::DepsDL
                | ReportType::AuditDL
                | ReportType::InfoJsonDL => {
                    let mut dir = config.report_path.clone();
                    dir.push(format!("{id}/{version}"));
                    match create_dir_all(&dir) {
                        Ok(()) => {
                            dir.push(&report_name);
                            match File::create(&dir) {
                                Ok(mut file) => match file.write(raw_str.as_bytes()) {
                                    Ok(_) => {
                                        events::wrote_report(id, version, config, report.into());
                                    }
                                    Err(_) => {
                                        eprintln!("Could not write to {report_name} file.");
                                    }
                                },
                                Err(_) => eprintln!("Could not create {report_name} file."),
                            }
                        }
                        Err(_) => {
                            eprintln!("Could not create directories for {report_name}.");
                        }
                    }
                }
                ReportType::LicenseEvent => events::print_license(id, version, raw_str),
                ReportType::DepsEvent => events::print_deps(id, version, raw_str),
                ReportType::AuditEvent => events::print_audit(id, version, raw_str),
                ReportType::InfoJsonEvent => events::print_info_json(id, version, raw_str),
            }
        }
    }

    fn fetch_latest(&mut self, id: &str) -> String {
        match self.interact.get_latest(id) {
            Ok(s) => s,
            Err(InteractError::Malformed) => panic!("The version string for {id} is malformed."),
            Err(InteractError::HttpCode(404)) => {
                panic!("Crate {id} {} in index!", color!(bright_red, "not found"))
            }
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
                color!(bright_red, "not found")
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
                color!(bright_red, "not found")
            ),
            Err(InteractError::HttpCode(code)) => {
                panic!("Http error {code} for {file} for {id}@{version}.")
            }
            Err(err) => panic!("Connection error.\n{err}"),
        }
    }

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

        assert!(
            !config.pub_keys.is_empty(),
            "{} for index '{}'. Please add one with --pub-key or use --no-verify.",
            color!(bright_red, "No public key(s)"),
            config.index
        );

        let sig = &self.fetch_str(id, version, sig_file);
        let signature = Signature::decode(sig).expect("Signature was malformed.");

        let mut verified = false;
        for key in &config.pub_keys {
            let pk = PublicKey::from_base64(key).expect("Public key was malformed.");
            if pk.verify(raw_file.as_bytes(), &signature, false).is_ok() {
                verified = true;
                break;
            }
        }

        if verified {
            eprintln!(
                "{} {file} for {id}@{version} with minisign.",
                color!(bright_white, "Verified")
            );
        }
        else {
            panic!(
                "{} verify {file} for {id}@{version}.",
                color!(bright_red, "Could not")
            );
        }

        verified
    }

    fn verify_archive(id: &str, version: &str, config: &Config, info: &InfoFileImm, bytes: &[u8]) {
        Self::verify_bytes(
            id,
            version,
            config,
            &info.archive_hashes,
            &format!("{} archive", &config.target),
            bytes,
        );
    }

    pub fn verify_binary(
        id: &str,
        version: &str,
        config: &Config,
        info: &InfoFileImm,
        binary_name: &str,
        bytes: &[u8],
    ) {
        Self::verify_bytes(
            id,
            version,
            config,
            info.bins_hashes
                .get(binary_name)
                .unwrap_or_else(|| panic!("{binary_name} is missing hashes.")),
            &format!("{} {binary_name} binary", &config.target),
            bytes,
        );
    }

    fn verify_bytes(
        id: &str,
        version: &str,
        config: &Config,
        in_hashes: &Hashes,
        item: &str,
        bytes: &[u8],
    ) {
        if config.no_hash {
            return;
        }

        {
            use sha3::{Digest, Sha3_256, Sha3_512};

            // sha3_512
            if let Some(sha_hash) = in_hashes.get(&HashType::Sha3_512) {
                let mut hasher = Sha3_512::new();
                hasher.update(bytes);
                let hash: Vec<u8> = hasher.finalize().to_vec();
                let hash = const_hex::encode(hash);

                assert!(
                    hash.eq(sha_hash),
                    "sha3_512 hashes do not match for {item}. {sha_hash} != {hash}"
                );

                eprintln!(
                    "{} {item} for {id}@{version} with sha3_512.",
                    color!(bright_white, "Verified")
                );
                return;
            }

            // sha3_256
            if let Some(sha_hash) = in_hashes.get(&HashType::Sha3_256) {
                let mut hasher = Sha3_256::new();
                hasher.update(bytes);
                let hash: Vec<u8> = hasher.finalize().to_vec();
                let hash = const_hex::encode(hash);

                assert!(
                    hash.eq(sha_hash),
                    "sha3_256 hashes do not match for {item}. {sha_hash} != {hash}"
                );

                eprintln!(
                    "{} {item} for {id}@{version} with sha3_256.",
                    color!(bright_white, "Verified")
                );
                return;
            }
        }

        {
            use sha2::{Digest, Sha256, Sha512};

            // sha512
            if let Some(sha_hash) = in_hashes.get(&HashType::Sha512) {
                let mut hasher = Sha512::new();
                hasher.update(bytes);
                let hash: Vec<u8> = hasher.finalize().to_vec();
                let hash = const_hex::encode(hash);

                assert!(
                    hash.eq(sha_hash),
                    "sha512 hashes do not match for {item}. {sha_hash} != {hash}"
                );

                eprintln!(
                    "{} {item} for {id}@{version} with sha512.",
                    color!(bright_white, "Verified")
                );
                return;
            }

            // sha256
            if let Some(sha_hash) = in_hashes.get(&HashType::Sha256) {
                let mut hasher = Sha256::new();
                hasher.update(bytes);
                let hash: Vec<u8> = hasher.finalize().to_vec();
                let hash = const_hex::encode(hash);

                assert!(
                    hash.eq(sha_hash),
                    "sha256 hashes do not match for {item}. {sha_hash} != {hash}"
                );

                eprintln!(
                    "{} {item} for {id}@{version} with sha256.",
                    color!(bright_white, "Verified")
                );
                return;
            }
        }

        eprintln!("Could not verify downloaded {item} for {id}@{version}.");
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
