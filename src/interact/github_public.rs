use crate::interact::Interact;
use ureq::Agent;

pub struct GithubPublic {
    agent: Agent,
    pre_url: String,
}
impl GithubPublic {
    pub fn new(agent: Agent, slug: &str) -> Self {
        let pre_url = format!("https://{slug}/releases/download");
        Self { agent, pre_url }
    }

    fn url(&self, id: &str, version: &str, file: &str) -> String {
        format!("{}/{id}-{version}/{file}", self.pre_url)
    }

    fn call(&self, url: &str) -> anyhow::Result<String> {
        let mut res = self.agent.get(url).call()?;
        let s = res.body_mut().read_to_string()?;
        Ok(s.trim().to_string())
    }
}
impl Interact for GithubPublic {
    fn get_latest(&mut self, id: &str) -> anyhow::Result<String> {
        let url = format!("{}/stable-index/{id}", self.pre_url);
        self.call(&url)
    }

    fn get_str(&mut self, id: &str, version: &str, file_name: &str) -> anyhow::Result<String> {
        let url = self.url(id, version, file_name);
        self.call(&url)
    }

    fn get_blob(&mut self, id: &str, version: &str, file_name: &str) -> anyhow::Result<Vec<u8>> {
        let url = self.url(id, version, file_name);

        let mut res = self.agent.get(&url).call()?;
        let bytes = res.body_mut().read_to_vec()?;

        Ok(bytes)
    }
}
