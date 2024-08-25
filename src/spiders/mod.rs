use crate::error::Error;
use async_trait::async_trait;

mod cvedetails;

pub use cvedetails::CveDetailsSpider;

#[async_trait]
pub trait Spider: Send + Sync {
    type Item: Send;
    fn start_urls(&self) -> Vec<String>;
    async fn scrape(&self, url: String) -> Result<(Vec<Self::Item>, Vec<String>), Error>;
    async fn process(&self, item: Self::Item) -> Result<(), Error>;
}

pub trait GetName: Spider {
    fn get_name() -> String;
}

pub fn get_spider_names() -> Vec<String> {
    vec![CveDetailsSpider::get_name()]
}

pub enum Spiders {
    CveDetails(CveDetailsSpider),
}

impl Spiders {
    pub fn inner(self) -> impl Spider + Send + Sync {
        match self {
            Spiders::CveDetails(s) => s,
        }
    }
}

impl std::convert::TryFrom<&str> for Spiders {
    type Error = crate::error::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if s == CveDetailsSpider::get_name() {
            Ok(Self::CveDetails(CveDetailsSpider::new()))
        } else {
            Err(Error::InvalidSpiderName(s.to_owned()))
        }
    }
}
