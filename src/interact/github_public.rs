use crate::interact::{Interact, InteractError};
use ureq::{Agent, Error};

pub struct GithubPublic {
    agent: Agent,
    slug: String,
}
impl GithubPublic {
    pub fn new(agent: Agent, slug: &str) -> Self {
        Self {
            agent,
            slug: slug.to_string(),
        }
    }

    fn pre_url(&self, id: &str, version: &str, file: &str) -> String {
        format!(
            "https://{}/releases/download/{id}-{version}/{file}",
            self.slug
        )
    }
}
impl Interact for GithubPublic {
    fn get_latest(&self, id: &str) -> Result<String, InteractError> {
        let url = format!("https://{}/releases/download/stable-index/{id}", self.slug);
        call(&self.agent, &url)
    }

    fn get_str(&self, id: &str, version: &str, file_name: &str) -> Result<String, InteractError> {
        let url = self.pre_url(id, version, file_name);
        call(&self.agent, &url)
    }

    fn get_blob(&self, id: &str, version: &str, file_name: &str) -> Result<Vec<u8>, InteractError> {
        let url = self.pre_url(id, version, file_name);
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

fn call(agent: &Agent, url: &str) -> Result<String, InteractError> {
    return match agent.get(url).call() {
        Ok(res) => {
            let s = res.into_string().map_err(|_| InteractError::Malformed)?;
            Ok(s.trim().to_string())
        }
        Err(Error::Status(code, _)) => Err(InteractError::HttpCode(code)),
        Err(_) => Err(InteractError::ConnectionError),
    };
}
