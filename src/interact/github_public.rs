use crate::interact::{Interact, InteractError};
use ureq::{Agent, Error};

pub struct GithubPublic {
    agent: Agent,
    pre_url: String,
}
impl GithubPublic {
    pub fn new(agent: Agent, slug: &str) -> Self {
        let pre_url = format!("https://{}/releases/download", slug);
        Self { agent, pre_url }
    }

    fn url(&self, id: &str, version: &str, file: &str) -> String {
        format!("{}/{id}-{version}/{file}", self.pre_url)
    }

    fn call(&self, url: &str) -> Result<String, InteractError> {
        return match self.agent.get(url).call() {
            Ok(res) => {
                let s = res.into_string().map_err(|_| InteractError::Malformed)?;
                Ok(s.trim().to_string())
            }
            Err(Error::Status(code, _)) => Err(InteractError::HttpCode(code)),
            Err(_) => Err(InteractError::ConnectionError),
        };
    }
}
impl Interact for GithubPublic {
    fn get_latest(&mut self, id: &str) -> Result<String, InteractError> {
        let url = format!("{}/stable-index/{id}", self.pre_url);
        self.call(&url)
    }

    fn get_str(&self, id: &str, version: &str, file_name: &str) -> Result<String, InteractError> {
        let url = self.url(id, version, file_name);
        self.call(&url)
    }

    fn get_blob(&self, id: &str, version: &str, file_name: &str) -> Result<Vec<u8>, InteractError> {
        let url = self.url(id, version, file_name);
        let mut bytes = Vec::new();
        match self.agent.get(&url).call() {
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

        Ok(bytes)
    }
}
