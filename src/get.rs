use std::{
    fs::{create_dir_all, File},
    io::Write,
};

use crate::{
    color,
    config::Config,
    data::{HashType, Hashes, HashesFile, HashesFileV1, InfoFile, InfoFileImm, Meta, ReportType},
    events,
    interact::{self, Interact},
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

    pub fn download_info(&mut self, meta: &Meta) -> InfoFileImm {
        eprintln!(
            "{} info for {}@{}...",
            color!(bright_blue, "Fetching"),
            meta.id,
            meta.version,
        );

        // info.json
        let raw_info_file = &self.fetch_str(meta.id, meta.version, "info.json");

        // info.json verify
        if !meta.config.no_sig {
            let v = self.verify_file(meta, "info.json", "info.json.minisig", raw_info_file);
            events::info_verify(meta, v);
        }

        let info: InfoFile = serde_json::from_str(raw_info_file)
            .unwrap_or_else(|_| panic!("info.json is malformed for {}@{}", meta.id, meta.version));
        let mut info: InfoFileImm = InfoFileImm::convert(info, &meta.config.target);

        assert!(
            info.id.eq(meta.id),
            "{}@{} does not match with info.json id {}",
            meta.id,
            meta.version,
            info.id
        );
        assert!(
            info.version.eq(meta.version),
            "{}@{} does not match with info.json version {}",
            meta.id,
            meta.version,
            info.version
        );

        // check if compression is supported
        assert!(
            info.archive.compression.eq("gz"),
            "{}@{} does not support compression gzip",
            meta.id,
            meta.version,
        );

        // check if binary does not exist if safe mode is on
        if meta.config.safe && !(meta.config.ci || meta.config.update) {
            for bin in &info.bins {
                let mut path = meta.config.path.clone();
                path.push(bin);

                assert!(
                    !path.exists(),
                    "Binary '{}' {} for {}@{}",
                    dunce::canonicalize(path)
                        .expect("Could not canonicalize path.")
                        .display(),
                    color!(bright_red, "already exists"),
                    meta.id,
                    meta.version,
                );
            }
        }

        if !meta.config.no_hash {
            if let Some(ref polyfill) = info.polyfill {
                eprintln!(
                    "{} hashes for {}@{} with target {}...",
                    color!(bright_blue, "Fetching"),
                    meta.id,
                    meta.version,
                    &meta.config.target
                );

                // hashes.json
                let raw_hashes_file = &self.fetch_str(meta.id, meta.version, &polyfill.hash_file);

                // hashes.json.minisig and test
                if !meta.config.no_sig {
                    if let Some(sig_file) = polyfill.hash_file_sig.clone() {
                        let v =
                            self.verify_file(meta, &polyfill.hash_file, &sig_file, raw_hashes_file);
                        events::hashes_verify(meta, v);
                    }
                    else {
                        panic!(
                            "Could not force sig for index {}. hashes.json is not signed for {}@{}.",
                            meta.config.index, meta.id, meta.version
                        );
                    }
                }

                let hashes: HashesFile =
                    serde_json::from_str(raw_hashes_file).unwrap_or_else(|_| {
                        panic!(
                            "{} is malformed for {}@{}",
                            polyfill.hash_file, meta.id, meta.version
                        )
                    });
                let hashes: HashesFileV1 = hashes.into();
                let hashes = hashes
                    .hashes
                    .get(&meta.config.target)
                    .unwrap_or_else(|| panic!("No hashes for target {}", meta.config.target));

                info.archive_hashes.clone_from(&hashes.archive);
                info.bins_hashes.clone_from(&hashes.bins);
            }
        }

        // check if target is supported, based on hash
        if !meta.config.no_hash {
            assert!(
                !info.archive_hashes.is_empty(),
                "{}@{} does {} target {}, due to empty archive hashes",
                meta.id,
                meta.version,
                color!(bright_red, "not support"),
                meta.config.target
            );
        }

        info
    }

    pub fn download_blob(&mut self, meta: &Meta, info: &InfoFileImm) -> Vec<u8> {
        // tar
        eprintln!(
            "{} {}@{} for target {}...",
            color!(bright_yellow, "Downloading"),
            meta.id,
            meta.version,
            &meta.config.target
        );
        let tar_bytes = self.fetch_blob(meta.id, meta.version, &info.archive_name);

        // test hashes
        Self::verify_archive(meta, info, &tar_bytes);

        tar_bytes
    }

    pub fn is_bin(info: &InfoFileImm, bin_name: &str) -> bool {
        let bin_name = bin_name.replace(".exe", "");
        info.bins.contains(&bin_name)
    }

    pub fn reports(&mut self, meta: &Meta, info: &InfoFileImm) {
        if meta.config.reports.is_empty() {
            return;
        }

        eprintln!("{} reports... ", color!(bright_blue, "Getting"));

        for report in &meta.config.reports {
            let report_name = match report {
                ReportType::LicenseDL | ReportType::LicenseEvent => info.files.license.clone(),
                ReportType::DepsDL | ReportType::DepsEvent => info.files.deps.clone(),
                ReportType::AuditDL | ReportType::AuditEvent => info.files.audit.clone(),
                ReportType::InfoJsonDL | ReportType::InfoJsonEvent => "info.json".to_string(),
            };

            let raw_str = &self.fetch_str(meta.id, meta.version, &report_name);

            match report {
                ReportType::LicenseDL
                | ReportType::DepsDL
                | ReportType::AuditDL
                | ReportType::InfoJsonDL => {
                    let mut dir = meta.config.report_path.clone();
                    dir.push(format!("{}/{}", meta.id, meta.version));
                    match create_dir_all(&dir) {
                        Ok(()) => {
                            dir.push(&report_name);
                            match File::create(&dir) {
                                Ok(mut file) => match file.write(raw_str.as_bytes()) {
                                    Ok(_) => {
                                        events::wrote_report(meta, report.into());
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
                ReportType::LicenseEvent => events::print_license(meta, raw_str),
                ReportType::DepsEvent => events::print_deps(meta, raw_str),
                ReportType::AuditEvent => events::print_audit(meta, raw_str),
                ReportType::InfoJsonEvent => events::print_info_json(meta, raw_str),
            }
        }
    }

    fn fetch_latest(&mut self, id: &str) -> String {
        self.interact.get_latest(id).unwrap()
    }

    fn fetch_str(&mut self, id: &str, version: &str, file: &str) -> String {
        self.interact.get_str(id, version, file).unwrap()
    }

    fn fetch_blob(&mut self, id: &str, version: &str, file: &str) -> Vec<u8> {
        self.interact.get_blob(id, version, file).unwrap()
    }

    fn verify_file(&mut self, meta: &Meta, file: &str, sig_file: &str, raw_file: &str) -> bool {
        use minisign_verify::{PublicKey, Signature};

        assert!(
            !meta.config.pub_keys.is_empty(),
            "{} for index '{}'. Please add one with --pub-key or use --no-verify.",
            color!(bright_red, "No public key(s)"),
            meta.config.index
        );

        let sig = &self.fetch_str(meta.id, meta.version, sig_file);
        let signature = Signature::decode(sig).expect("Signature was malformed.");

        let mut verified = false;
        for key in &meta.config.pub_keys {
            let pk = PublicKey::from_base64(key).expect("Public key was malformed.");
            if pk.verify(raw_file.as_bytes(), &signature, false).is_ok() {
                verified = true;
                break;
            }
        }

        if verified {
            eprintln!(
                "{} {file} for {}@{} with minisign.",
                color!(bright_white, "Verified"),
                meta.id,
                meta.version
            );
        }
        else {
            panic!(
                "{} verify {file} for {}@{}.",
                color!(bright_red, "Could not"),
                meta.id,
                meta.version
            );
        }

        verified
    }

    fn verify_archive(meta: &Meta, info: &InfoFileImm, bytes: &[u8]) {
        Self::verify_bytes(
            meta,
            &info.archive_hashes,
            &format!("{} archive", &meta.config.target),
            bytes,
        );
    }

    pub fn verify_binary(meta: &Meta, info: &InfoFileImm, binary_name: &str, bytes: &[u8]) {
        Self::verify_bytes(
            meta,
            info.bins_hashes
                .get(binary_name)
                .unwrap_or_else(|| panic!("{binary_name} is missing hashes.")),
            &format!("{} {binary_name} binary", &meta.config.target),
            bytes,
        );
    }

    fn verify_bytes(meta: &Meta, in_hashes: &Hashes, item: &str, bytes: &[u8]) {
        if meta.config.no_hash {
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
                    "{} {item} for {}@{} with sha3_512.",
                    color!(bright_white, "Verified"),
                    meta.id,
                    meta.version
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
                    "{} {item} for {}@{} with sha3_256.",
                    color!(bright_white, "Verified"),
                    meta.id,
                    meta.version
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
                    "{} {item} for {}@{} with sha512.",
                    color!(bright_white, "Verified"),
                    meta.id,
                    meta.version
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
                    "{} {item} for {}@{} with sha256.",
                    color!(bright_white, "Verified"),
                    meta.id,
                    meta.version
                );
                return;
            }
        }

        eprintln!(
            "Could not verify downloaded {item} for {}@{}.",
            meta.id, meta.version
        );
    }

    pub fn verify_bytes_update(in_hashes: &Hashes, item: &str, bytes: &[u8]) -> bool {
        {
            use sha3::{Digest, Sha3_256, Sha3_512};

            // sha3_512
            if let Some(sha_hash) = in_hashes.get(&HashType::Sha3_512) {
                let mut hasher = Sha3_512::new();
                hasher.update(bytes);
                let hash: Vec<u8> = hasher.finalize().to_vec();
                let hash = const_hex::encode(hash);

                if !hash.eq(sha_hash) {
                    eprintln!(
                        "Update: sha3_512 hashes do not match for {item}. {sha_hash} != {hash}"
                    );
                    return false;
                }
                return true;
            }

            // sha3_256
            if let Some(sha_hash) = in_hashes.get(&HashType::Sha3_256) {
                let mut hasher = Sha3_256::new();
                hasher.update(bytes);
                let hash: Vec<u8> = hasher.finalize().to_vec();
                let hash = const_hex::encode(hash);

                if !hash.eq(sha_hash) {
                    eprintln!(
                        "Update: sha3_256 hashes do not match for {item}. {sha_hash} != {hash}"
                    );
                    return false;
                }
                return true;
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

                if !hash.eq(sha_hash) {
                    eprintln!(
                        "Update: sha512 hashes do not match for {item}. {sha_hash} != {hash}"
                    );
                    return false;
                }
                return true;
            }

            // sha256
            if let Some(sha_hash) = in_hashes.get(&HashType::Sha256) {
                let mut hasher = Sha256::new();
                hasher.update(bytes);
                let hash: Vec<u8> = hasher.finalize().to_vec();
                let hash = const_hex::encode(hash);

                if !hash.eq(sha_hash) {
                    eprintln!(
                        "Update: sha256 hashes do not match for {item}. {sha_hash} != {hash}"
                    );
                    return false;
                }
                return true;
            }
        }

        false
    }
}
