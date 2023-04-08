use ureq::Agent;

#[cfg(feature = "github-public")]
mod github_public;

#[derive(thiserror::Error, Debug)]
pub enum InteractError {
    #[error("The received string is malformed.")]
    Malformed,
    #[error("Http code `{0}`")]
    HttpCode(u16),
    #[error("Connection error")]
    ConnectionError,
    // #[error("Unknown error")]
    // Unknown,
}

pub fn create_interact(
    input: Option<&str>,
    _auth: Option<&str>,
    agent: Agent,
) -> Box<dyn Interact> {
    if input.is_none() {
        #[cfg(feature = "github-public")]
        {
            return Box::new(github_public::GithubPublic::new(
                agent,
                "github.com/crow-rest/cargo-prebuilt-index",
            ));
        }
        #[cfg(not(feature = "github-public"))]
        {
            println!("Using the default index requires the github-public feature!");
            std::process::exit(-220);
        }
    }

    todo!()
}

pub trait Interact {
    fn pre_url(&self, id: &str, version: &str, target: &str) -> String;

    fn get_latest(&self, id: &str) -> Result<String, InteractError>;
    fn get_hash(&self, id: &str, version: &str, target: &str) -> Result<String, InteractError>;
    fn get_tar(&self, id: &str, version: &str, target: &str) -> Result<Vec<u8>, InteractError>;
    fn get_report(&self, id: &str, version: &str, name: &str) -> Result<String, InteractError>;
}
