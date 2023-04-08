use crate::interact::{Interact, InteractError};
use ureq::{Agent, Error, Request};

pub struct GithubPublic {
    slug: String,
}
impl GithubPublic {
    pub fn new(slug: &str) -> Self {
        Self {
            slug: slug.to_string(),
        }
    }
}
impl Interact for GithubPublic {
    fn pre_url(&self, id: &String, version: &String, target: &String) -> String {
        format!(
            "https://{}/releases/download/{id}-{version}/{target}",
            self.slug
        )
    }

    fn get_latest(&self, agent: &Agent, id: &String) -> Result<String, InteractError> {
        let url = format!("https://{}/releases/download/stable-index/{id}", self.slug);
        call(agent, &url)
    }

    fn get_hash(
        &self,
        agent: &Agent,
        id: &String,
        version: &String,
        target: &String,
    ) -> Result<String, InteractError> {
        let url = format!("{}.sha256", self.pre_url(id, version, target));
        call(agent, &url)
    }

    fn get_tar(
        &self,
        agent: &Agent,
        id: &String,
        version: &String,
        target: &String,
    ) -> Result<Vec<u8>, InteractError> {
        let url = format!("{}.tar.gz", self.pre_url(id, version, target));
        let mut bytes = Vec::new();
        match agent.get(&url).call() {
            Ok(response) => {
                response
                    .into_reader()
                    .read_to_end(&mut bytes)
                    .map_err(|_| InteractError::Malformed)?;
            }
            Err(Error::Status(code, _)) => return Err(InteractError::HttpCode(code)),
            Err(_) => return Err(InteractError::ConnectionError),
        }

        return Ok(bytes);
    }

    fn get_report(
        &self,
        agent: &Agent,
        id: &String,
        version: &String,
        name: &String,
    ) -> Result<String, InteractError> {
        let url = format!(
            "https://{}/releases/download/{id}-{version}/{name}.report",
            self.slug
        );
        call(agent, &url)
    }
}

fn call(agent: &Agent, url: &String) -> Result<String, InteractError> {
    return match agent.get(url).call() {
        Ok(res) => {
            let s = res.into_string().map_err(|_| InteractError::Malformed)?;
            Ok(s.trim().to_string())
        }
        Err(Error::Status(code, _)) => Err(InteractError::HttpCode(code)),
        Err(_) => Err(InteractError::ConnectionError),
    };
}
