use crate::interact::{Interact, InteractError};
use ureq::{Agent, Error};

pub struct GithubPrivate {
    agent: Agent,
    slug: String,
    auth_token: String,
}
impl GithubPrivate {
    pub fn new(agent: Agent, auth_token: String, slug: &str) -> Self {
        Self {
            agent,
            auth_token,
            slug: slug.to_string(),
        }
    }
}
impl Interact for GithubPrivate {
    fn get_latest(&self, id: &str) -> Result<String, InteractError> {
        todo!()
    }
    fn get_str(&self, id: &str, version: &str, file_name: &str) -> Result<String, InteractError> {
        todo!()
    }
    fn get_blob(&self, id: &str, version: &str, file_name: &str) -> Result<Vec<u8>, InteractError> {
        todo!()
    }
}
