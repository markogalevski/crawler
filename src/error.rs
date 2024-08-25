use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Incorrect usage")]
    CliUsage,
    #[error("Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Invalid spider name provided {0}")]
    InvalidSpiderName(String),
}
