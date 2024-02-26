use ureq::Agent;

use crate::color::{err_color_print, PossibleColor};

#[cfg(feature = "github-private")]
mod github_private;
#[cfg(feature = "github-public")]
mod github_public;

#[derive(Debug)]
pub enum InteractError {
    Malformed,
    HttpCode(u16),
    ConnectionError,
    // #[error("Unknown error")]
    // Unknown,
}
impl std::fmt::Display for InteractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malformed => write!(f, "The received string is malformed."),
            Self::HttpCode(code) => write!(f, "Http code {code}"),
            Self::ConnectionError => write!(f, "Connection error"),
        }
    }
}
impl std::error::Error for InteractError {}

pub fn create_interactive(input: &str, auth: Option<&String>, agent: Agent) -> Box<dyn Interact> {
    // Github public
    if input.starts_with("gh-pub:") {
        #[cfg(feature = "github-public")]
        {
            let url = input
                .get(7..input.len())
                .expect("Missing url after gh-pub:");
            eprintln!(
                "{} index https://{url}.",
                err_color_print("Using", &PossibleColor::BrightCyan),
            );
            return Box::new(github_public::GithubPublic::new(agent, url));
        }
        #[cfg(not(feature = "github-public"))]
        panic!("Using this index ({input}) requires the github-public feature!");
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
                err_color_print("Using", &PossibleColor::BrightCyan),
            );
            return Box::new(github_private::GithubPrivate::new(
                agent,
                auth.expect("Need auth token for private index.").clone(),
                url,
            ));
        }
        #[cfg(not(feature = "github-private"))]
        panic!("Using this index ({input}) requires the github-private feature!");
    }

    panic!("This index ({input}) is not supported or malformed.");
}

pub trait Interact {
    fn get_latest(&mut self, id: &str) -> Result<String, InteractError>;
    fn get_str(
        &mut self,
        id: &str,
        version: &str,
        file_name: &str,
    ) -> Result<String, InteractError>;
    fn get_blob(
        &mut self,
        id: &str,
        version: &str,
        file_name: &str,
    ) -> Result<Vec<u8>, InteractError>;
}
