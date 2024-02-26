use crate::interact::{Interact, InteractError};
use serde::{de::DeserializeOwned, Deserialize};
use std::collections::HashMap;
use ureq::{Agent, Error};

#[derive(Clone, Debug, Deserialize)]
struct Release {
    assets: Vec<ReleaseAssets>,
}

#[derive(Clone, Debug, Deserialize)]
struct ReleaseAssets {
    url: String,
    name: String,
}

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
        assert_eq!(s.len(), 3, "Slug '{slug}' is not formatted properly.");

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

    fn get_str_file(
        &self,
        assets: &Vec<ReleaseAssets>,
        file: &str,
    ) -> Result<String, InteractError> {
        let mut val = None;
        for i in assets {
            if i.name.eq(file) {
                let latest = self.call(&i.url)?;
                val = Some(latest);
                break;
            }
        }

        val.ok_or(InteractError::HttpCode(404))
    }

    fn get_release(&mut self, id: &str, version: &str) -> Result<Release, InteractError> {
        let key = format!("{id}/--/{version}");

        if let Some(item) = self.index.get(&key) {
            Ok(item.clone())
        }
        else {
            let rel = self.api_call(&format!(
                "{}/repos/{}/{}/releases/tags/{id}-{version}",
                self.u_url, self.u_owner, self.u_repo
            ))?;
            let _ = self.index.insert(key.clone(), rel);
            Ok(self.index.get(&key).unwrap().clone())
        }
    }
}
impl Interact for GithubPrivate {
    fn get_latest(&mut self, id: &str) -> Result<String, InteractError> {
        if self.stable_index.is_none() {
            self.stable_index = Some(self.api_call(&format!(
                "{}/repos/{}/{}/releases/tags/stable-index",
                self.u_url, self.u_owner, self.u_repo
            ))?);
        }

        // Get latest from file
        let si = self
            .stable_index
            .as_ref()
            .expect("Should have stable index!");

        self.get_str_file(&si.assets, id)
    }

    fn get_str(
        &mut self,
        id: &str,
        version: &str,
        file_name: &str,
    ) -> Result<String, InteractError> {
        let release = self.get_release(id, version)?;
        self.get_str_file(&release.assets, file_name)
    }

    fn get_blob(
        &mut self,
        id: &str,
        version: &str,
        file_name: &str,
    ) -> Result<Vec<u8>, InteractError> {
        let release = self.get_release(id, version)?;

        let mut val = None;
        for i in &release.assets {
            if i.name.eq(file_name) {
                let mut bytes = Vec::new();
                match self
                    .agent
                    .get(&i.url)
                    .set("Accept", "application/octet-stream")
                    .set(
                        "Authorization",
                        format!("Bearer {}", self.auth_token).as_str(),
                    )
                    .call()
                {
                    Ok(response) => {
                        //TODO: Allow limiting of size.
                        response
                            .into_reader()
                            .read_to_end(&mut bytes)
                            .map_err(|_| InteractError::Malformed)?;
                    }
                    Err(Error::Status(code, _)) => return Err(InteractError::HttpCode(code)),
                    Err(_) => return Err(InteractError::ConnectionError),
                }

                val = Some(bytes);
                break;
            }
        }

        val.ok_or(InteractError::HttpCode(404))
    }
}
