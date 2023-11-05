use crate::interact::{Interact, InteractError};
use serde::{de::DeserializeOwned, Deserialize};
use std::collections::HashMap;
use ureq::{Agent, Error};

#[allow(unused)] // TODO: REMOVE
pub struct GithubPrivate {
    agent: Agent,
    auth_token: String,
    u_url: String,
    u_owner: String,
    u_repo: String,
    stable_index: Option<Release>,
    index: HashMap<String, Release>,
}
impl GithubPrivate {
    pub fn new(agent: Agent, auth_token: String, slug: &str) -> Self {
        let s: Vec<&str> = slug.split('/').collect();
        if s.len() != 3 {
            panic!("Slug '{slug}' is not formatted properly.");
        }

        Self {
            agent,
            auth_token,
            u_url: format!("https://api.{}", s[0]),
            u_owner: s[1].to_string(),
            u_repo: s[2].to_string(),
            stable_index: None,
            index: HashMap::new(),
        }
    }

    fn api_call<T: DeserializeOwned>(&self, url: &str) -> Result<T, InteractError> {
        return match self
            .agent
            .get(url)
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28")
            .set(
                "Authorization",
                format!("Bearer {}", self.auth_token).as_str(),
            )
            .call()
        {
            Ok(res) => {
                let s = res.into_string().map_err(|_| InteractError::Malformed)?;
                let json = serde_json::from_str(&s)
                    .unwrap_or_else(|_| panic!("Could not parse api json from {url}"));
                Ok(json)
            }
            Err(Error::Status(code, _)) => Err(InteractError::HttpCode(code)),
            Err(_) => Err(InteractError::ConnectionError),
        };
    }

    fn call(&self, url: &str) -> Result<String, InteractError> {
        return match self
            .agent
            .get(url)
            .set("Accept", "application/octet-stream")
            .set(
                "Authorization",
                format!("Bearer {}", self.auth_token).as_str(),
            )
            .call()
        {
            Ok(res) => {
                let s = res.into_string().map_err(|_| InteractError::Malformed)?;
                Ok(s.trim().to_string())
            }
            Err(Error::Status(code, _)) => Err(InteractError::HttpCode(code)),
            Err(_) => Err(InteractError::ConnectionError),
        };
    }
}
impl Interact for GithubPrivate {
    fn get_latest(&mut self, id: &str) -> Result<String, InteractError> {
        if self.stable_index.is_none() {
            self.stable_index = Some(self.api_call(&format!(
                "{}/repos/{}/{}/releases/tags/stable-index",
                self.u_url, self.u_owner, self.u_repo
            ))?)
        }

        // Get latest from file
        let si = self
            .stable_index
            .as_ref()
            .expect("Should have stable index!");

        let mut val = None;
        for i in &si.assets {
            if i.name.eq(id) {
                let latest = self.call(&i.url)?;
                val = Some(latest);
                break;
            }
        }

        match val {
            Some(str) => Ok(str),
            None => Err(InteractError::HttpCode(404)),
        }
    }

    // TODO: This and blob need to be mutable since they might change the state of 'index'
    #[allow(unused)] // TODO: REMOVE
    fn get_str(
        &mut self,
        id: &str,
        version: &str,
        file_name: &str,
    ) -> Result<String, InteractError> {
        todo!()
    }

    #[allow(unused)] // TODO: REMOVE
    fn get_blob(
        &mut self,
        id: &str,
        version: &str,
        file_name: &str,
    ) -> Result<Vec<u8>, InteractError> {
        todo!()
    }
}

#[allow(unused)] // TODO: REMOVE
#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<ReleaseAssets>,
}

#[allow(unused)] // TODO: REMOVE
#[derive(Debug, Deserialize)]
struct ReleaseAssets {
    url: String,
    browser_download_url: String,
    name: String,
    state: String,
    size: u64,
}
