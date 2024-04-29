use serde_json::json;
use std::path::Path;

use crate::config::Config;

static EVENT_VERSION: &str = "1";

fn event(id: &str, version: &str, event: &str, data: &str) {
    println!(
        "{}",
        serde_json::to_string(&json!({
            "crate": id,
            "version": version,
            "event_version": EVENT_VERSION,
            "event": event,
            "data": data,
        }))
        .unwrap_or_else(|_| "Could not generate {event} event.".to_string())
    );
}

pub fn info_verify(id: &str, version: &str, config: &Config, verified: bool) {
    if config.out {
        event(id, version, "info_verified", &verified.to_string());
    }
}

pub fn hashes_verify(id: &str, version: &str, config: &Config, verified: bool) {
    if config.out {
        event(id, version, "hashes_verified", &verified.to_string());
    }
}

pub fn target(id: &str, version: &str, config: &Config) {
    if config.out {
        event(id, version, "target", &config.target);
    }
}

pub fn binary_installed(id: &str, version: &str, config: &Config, path: &Path) {
    if config.out {
        let path = format!("{path:?}");
        let mut path = path.as_str();
        path = path.strip_prefix('"').unwrap_or(path);
        path = path.strip_suffix('"').unwrap_or(path);

        event(id, version, "bin_installed", path);
    }
}

pub fn installed(id: &str, version: &str, config: &Config) {
    if config.out {
        event(id, version, "installed", &format!("{id}@{version}"));
    }
}

pub fn wrote_report(id: &str, version: &str, config: &Config, report_type: &str) {
    if config.out {
        event(id, version, "wrote_report", report_type);
    }
}

pub fn print_license(id: &str, version: &str, text: &str) {
    event(id, version, "print_license", text);
}

pub fn print_info_json(id: &str, version: &str, text: &str) {
    event(id, version, "print_info_json", text);
}

pub fn print_deps(id: &str, version: &str, text: &str) {
    event(id, version, "print_deps", text);
}

pub fn print_audit(id: &str, version: &str, text: &str) {
    event(id, version, "print_audit", text);
}

pub fn get_latest(id: &str, version: &str) {
    event(id, version, "latest_version", version);
}
