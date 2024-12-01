use crate::interact::Interact;
use serde::{de::DeserializeOwned, Deserialize};
use std::collections::HashMap;
use ureq::Agent;

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

    fn api_call<T: DeserializeOwned>(&self, url: &str) -> anyhow::Result<T> {
        let mut res = self
            .agent
            .get(url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header(
                "Authorization",
                format!("Bearer {}", self.auth_token).as_str(),
            )
            .call()?;

        let s = res.body_mut().read_to_string()?;
        let json = serde_json::from_str(&s)
            .unwrap_or_else(|_| panic!("Could not parse api json from {url}"));
        Ok(json)
    }

    fn call(&self, url: &str) -> anyhow::Result<String> {
        let mut res = self
            .agent
            .get(url)
            .header("Accept", "application/octet-stream")
            .header(
                "Authorization",
                format!("Bearer {}", self.auth_token).as_str(),
            )
            .call()?;

        let s = res.body_mut().read_to_string()?;
        Ok(s.trim().to_string())
    }

    fn get_str_file(&self, assets: &Vec<ReleaseAssets>, file: &str) -> anyhow::Result<String> {
        let mut val = None;
        for i in assets {
            if i.name.eq(file) {
                let latest = self.call(&i.url)?;
                val = Some(latest);
                break;
            }
        }

        val.ok_or_else(|| anyhow::anyhow!("Could not find {file} in assets list."))
    }

    fn get_release(&mut self, id: &str, version: &str) -> anyhow::Result<Release> {
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
    fn get_latest(&mut self, id: &str) -> anyhow::Result<String> {
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

    fn get_str(&mut self, id: &str, version: &str, file_name: &str) -> anyhow::Result<String> {
        let release = self.get_release(id, version)?;
        self.get_str_file(&release.assets, file_name)
    }

    fn get_blob(&mut self, id: &str, version: &str, file_name: &str) -> anyhow::Result<Vec<u8>> {
        let release = self.get_release(id, version)?;

        let mut val = None;
        for i in &release.assets {
            if i.name.eq(file_name) {
                let mut res = self
                    .agent
                    .get(&i.url)
                    .header("Accept", "application/octet-stream")
                    .header(
                        "Authorization",
                        format!("Bearer {}", self.auth_token).as_str(),
                    )
                    .call()?;

                let bytes = res.body_mut().read_to_vec()?;

                val = Some(bytes);
                break;
            }
        }

        val.ok_or_else(|| anyhow::anyhow!("Could not find {file_name} in assets list."))
    }
}
