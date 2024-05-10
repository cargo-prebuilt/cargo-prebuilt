use serde_json::json;

use crate::data::Meta;

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

pub fn info_verify(meta: &Meta, verified: bool) {
    if meta.config.out {
        event(
            meta.id,
            meta.version,
            "info_verified",
            &verified.to_string(),
        );
    }
}

pub fn hashes_verify(meta: &Meta, verified: bool) {
    if meta.config.out {
        event(
            meta.id,
            meta.version,
            "hashes_verified",
            &verified.to_string(),
        );
    }
}

pub fn target(meta: &Meta) {
    if meta.config.out {
        event(meta.id, meta.version, "target", &meta.config.target);
    }
}

pub fn binary_installed(meta: &Meta, path: &str) {
    if meta.config.out {
        event(meta.id, meta.version, "bin_installed", path);
    }
}

pub fn installed(meta: &Meta) {
    if meta.config.out {
        event(
            meta.id,
            meta.version,
            "installed",
            &format!("{}@{}", meta.id, meta.version),
        );
    }
}

pub fn wrote_report(meta: &Meta, report_type: &str) {
    if meta.config.out {
        event(meta.id, meta.version, "wrote_report", report_type);
    }
}

pub fn print_license(meta: &Meta, text: &str) {
    event(meta.id, meta.version, "print_license", text);
}

pub fn print_info_json(meta: &Meta, text: &str) {
    event(meta.id, meta.version, "print_info_json", text);
}

pub fn print_deps(meta: &Meta, text: &str) {
    event(meta.id, meta.version, "print_deps", text);
}

pub fn print_audit(meta: &Meta, text: &str) {
    event(meta.id, meta.version, "print_audit", text);
}

pub fn get_latest(id: &str, version: &str) {
    event(id, version, "latest_version", version);
}
