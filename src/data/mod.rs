mod config;
mod hashes;
mod info;

pub use config::*;
pub use hashes::*;
pub use info::*;

use crate::config::Config;

pub struct Meta<'a> {
    pub id: &'a str,
    pub version: &'a str,
    pub config: &'a Config,
}
impl<'a> Meta<'a> {
    pub const fn new(id: &'a str, version: &'a str, config: &'a Config) -> Self {
        Self {
            id,
            version,
            config,
        }
    }
}
