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
    fn pre_url(&self, id: &str, version: &str, target: &str) -> String {
        format!(
            "https://{}/releases/download/{id}-{version}/{target}",
            self.slug
        )
    }

    fn get_latest(&self, id: &str) -> Result<String, InteractError> {
        let url = format!("https://{}/releases/download/stable-index/{id}", self.slug);
        call(&self.agent, &url)
    }

    fn get_hash(&self, id: &str, version: &str, target: &str) -> Result<String, InteractError> {
        let url = format!("{}.sha256", self.pre_url(id, version, target));
        call(&self.agent, &url)
    }

    fn get_tar(&self, id: &str, version: &str, target: &str) -> Result<Vec<u8>, InteractError> {
        let url = format!("{}.tar.gz", self.pre_url(id, version, target));
        let mut bytes = Vec::new();
        match self.agent.get(&url).call() {
            Ok(response) => {
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

    fn get_report(&self, id: &str, version: &str, name: &str) -> Result<String, InteractError> {
        let url = format!(
            "https://{}/releases/download/{id}-{version}/{name}.report",
            self.slug
        );
        call(&self.agent, &url)
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
