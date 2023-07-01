use owo_colors::{OwoColorize, Stream::Stderr};
use ureq::Agent;

#[cfg(feature = "github-private")]
mod github_private;
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

// TODO: Remove _ from auth.
pub fn create_interact(input: String, _auth: Option<&String>, agent: Agent) -> Box<dyn Interact> {
    // Github public
    if input.starts_with("gh-pub:") {
        #[cfg(feature = "github-public")]
        {
            let url = input
                .get(7..input.len())
                .expect("Missing url after gh-pub:");
            eprintln!(
                "{} index https://{url}.",
                "Using".if_supports_color(Stderr, |text| text.bright_cyan())
            );
            return Box::new(github_public::GithubPublic::new(agent, url));
        }
        #[cfg(not(feature = "github-public"))]
        {
            eprintln!("Using this index ({input}) requires the github-public feature!");
            std::process::exit(220);
        }
    }

    // Github private
    if input.starts_with("gh-pri:") {
        #[cfg(feature = "github-private")]
        {
            let url = input
                .get(7..input.len())
                .expect("Missing url after gh-pri:");
            eprintln!(
                "{} index https://{url}.",
                "Using".if_supports_color(Stderr, |text| text.bright_cyan())
            );
            return Box::new(github_private::GithubPrivate::new(
                agent,
                _auth.expect("Need auth token for private index.").clone(),
                url,
            ));
        }
        #[cfg(not(feature = "github-private"))]
        {
            eprintln!("Using this index ({input}) requires the github-private feature!");
            std::process::exit(220);
        }
    }

    eprintln!("This index ({input}) is not supported or malformed.");
    std::process::exit(221);
}

pub trait Interact {
    fn get_latest(&self, id: &str) -> Result<String, InteractError>;
    fn get_str(&self, id: &str, version: &str, file_name: &str) -> Result<String, InteractError>;
    fn get_blob(&self, id: &str, version: &str, file_name: &str) -> Result<Vec<u8>, InteractError>;
}
