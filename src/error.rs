use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Incorrect usage")]
    CliUsage,
    #[error("Reqwest: {0}")]
    Reqest(#[from] reqwest::Error),
}
