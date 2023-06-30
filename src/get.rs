use crate::{
    config::{Config, REPORT_FLAGS},
    interact::{self, Interact, InteractError},
};
use owo_colors::{OwoColorize, Stream::Stdout};
use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
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

    pub fn get_latest(&self, id: &str) -> String {
        todo!()
    }
}

pub fn latest_version(interact: &dyn Interact, id: &str) -> String {
    match interact.get_latest(id) {
        Ok(s) => s,
        Err(InteractError::Malformed) => {
            eprintln!("The version string for {id} is malformed.");
            std::process::exit(342);
        }
        Err(InteractError::HttpCode(404)) => {
            eprintln!(
                "Crate {id} {} in index!",
                "not found".if_supports_color(Stdout, |text| text.bright_red())
            );
            std::process::exit(3);
        }
        Err(InteractError::HttpCode(code)) => {
            eprintln!("Http error {code} for crate {id}.");
            std::process::exit(4);
        }
        Err(_) => {
            eprintln!("Connection error.");
            std::process::exit(5);
        }
    }
}

pub fn hash(interact: &dyn Interact, id: &str, version: &str, target: &str) -> String {
    match interact.get_hash(id, version, target) {
        Ok(s) => s,
        Err(InteractError::Malformed) => {
            eprintln!("The hash string for {id} is malformed.");
            std::process::exit(343);
        }
        Err(InteractError::HttpCode(404)) => {
            eprintln!(
                "Crate {id}, version {version}, and target {target} was {}! (Hash)",
                "not found".if_supports_color(Stdout, |text| text.bright_red())
            );
            std::process::exit(9);
        }
        Err(InteractError::HttpCode(code)) => {
            eprintln!("Http error {code} for crate {id}.");
            std::process::exit(10);
        }
        Err(_) => {
            eprintln!("Connection error.");
            std::process::exit(11);
        }
    }
}

pub fn tar(interact: &dyn Interact, id: &str, version: &str, target: &str) -> Vec<u8> {
    println!(
        "{} {id} {version} from {}.tar.gz",
        "Downloading".if_supports_color(Stdout, |text| text.bright_blue()),
        interact.pre_url(id, version, target)
    );

    match interact.get_tar(id, version, target) {
        Ok(b) => b,
        Err(InteractError::Malformed) => {
            eprintln!("The tar bytes for {id} are malformed.");
            std::process::exit(344);
        }
        Err(InteractError::HttpCode(404)) => {
            eprintln!(
                "Crate {id}, version {version}, and target {target} was {}! (Tar)",
                "not found".if_supports_color(Stdout, |text| text.bright_red())
            );
            std::process::exit(6);
        }
        Err(InteractError::HttpCode(code)) => {
            eprintln!("Http error {code} for crate {id}.");
            std::process::exit(7);
        }
        Err(_) => {
            eprintln!("Connection error.");
            std::process::exit(8);
        }
    }
}

pub fn reports(interact: &dyn Interact, args: &Config, path: &Path, id: &str, version: &str) {
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

        let mut report_path = path.to_path_buf();
        report_path.push(format!(".prebuilt/reports/{id}/{version}"));
        let report_path = report_path;

        // license.report
        handle_report(
            interact,
            id,
            version,
            "license",
            &report_path,
            license_out,
            license_dl,
        );
        // deps.report
        handle_report(
            interact,
            id,
            version,
            "deps",
            &report_path,
            deps_out,
            deps_dl,
        );
        // audit.report
        handle_report(
            interact,
            id,
            version,
            "audit",
            &report_path,
            audit_out,
            audit_dl,
        );
    }
}

fn handle_report(
    interact: &dyn Interact,
    id: &str,
    version: &str,
    name: &str,
    report_path: &Path,
    out: bool,
    dl: bool,
) {
    if out || dl {
        let report = match interact.get_report(id, version, name) {
            Ok(r) => r,
            Err(InteractError::HttpCode(404)) => {
                eprintln!("Could not find a {name} report in the index.");
                return;
            }
            Err(_) => {
                eprintln!("Unknown error when trying to get {name} report.");
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
                                eprintln!("Could not write to {name}.report file.")
                            }
                        },
                        Err(_) => eprintln!("Could not create {name}.report file."),
                    }
                }
                Err(_) => {
                    eprintln!("Could not create directories for {name}.report.")
                }
            }
        }
    }
}
