use ureq::Agent;

use crate::color;

#[cfg(feature = "github-private")]
mod github_private;
#[cfg(feature = "github-public")]
mod github_public;

pub fn create_interactive(input: &str, auth: Option<&String>, agent: Agent) -> Box<dyn Interact> {
    // Github public
    if input.starts_with("gh-pub:") {
        #[cfg(feature = "github-public")]
        {
            let url = input
                .get(7..input.len())
                .expect("Missing url after gh-pub:");
            eprintln!("{} index https://{url}.", color!(bright_cyan, "Using"));
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
            eprintln!("{} index https://{url}.", color!(bright_cyan, "Using"));
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
    fn get_latest(&mut self, id: &str) -> anyhow::Result<String>;
    fn get_str(&mut self, id: &str, version: &str, file_name: &str) -> anyhow::Result<String>;
    fn get_blob(&mut self, id: &str, version: &str, file_name: &str) -> anyhow::Result<Vec<u8>>;
}
