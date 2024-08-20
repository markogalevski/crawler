use std::time::Duration;

use crate::error::Error;
use async_trait::async_trait;
use reqwest::{Client, ClientBuilder};
use select::{
    document::Document,
    node::Node,
    predicate::{Attr, Class, Element, Name, Predicate, Text},
};

use super::Spider;
const CVE_DETAILS_COM: &'static str = "http://www.cvedetails.com";

#[derive(Debug, Clone)]
pub struct Cve {
    name: String,
    url: String,
    max_cvss: Option<f32>,
    epss: Option<f32>,
    publish_date: String,
    update_date: String,
}

impl Cve {
    pub fn new(
        name: String,
        url: String,
        max_cvss: String,
        epss: String,
        publish_date: String,
        update_date: String,
    ) -> Self {
        let max_cvss = max_cvss.parse().ok();
        let epss = epss.parse().ok();
        Self {
            name,
            url,
            max_cvss,
            epss,
            publish_date,
            update_date,
        }
    }
}

pub struct CveDetailsSpider {
    http_client: Client,
}

impl CveDetailsSpider {
    pub fn new() -> Self {
        Self {
            http_client: ClientBuilder::new()
                .timeout(Duration::from_secs(5))
                .build()
                .expect("Building http client failed"),
        }
    }
}
fn normalise_link(segment: &str) -> String {
    format!("{CVE_DETAILS_COM}/{segment}")
}

#[async_trait]
impl Spider for CveDetailsSpider {
    type Item = Cve;

    fn name(&self) -> String {
        "CveDetails".to_owned()
    }

    fn start_urls(&self) -> Vec<String> {
        todo!()
    }

    async fn scrape(&self, url: String) -> Result<(Vec<Self::Item>, Vec<String>), Error> {
        log::info!("visiting: {url}");
        let http_res = self.http_client.get(&url).send().await?.text().await?;
        let mut items = vec![];

        let document = Document::from(http_res.as_str());
        let rows =
            document.find(Attr("id", "searchresults").descendant(Attr("data-tsvfield", "cveinfo")));
        for row in rows {
            let name = row
                .find(Attr("data-tsvfield", "cveId"))
                .nth(0)
                .unwrap()
                .text();
            let url = row
                .find(Attr("data-tsvfield", "cveId").child(Name("a")))
                .nth(0)
                .map(|a| a.attr("href"))
                .flatten()
                .map(normalise_link)
                .unwrap();
            let max_cvss = row
                .find(Attr("data-tsvfield", "maxCvssBaseScore").child(Name("div")))
                .nth(0)
                .unwrap()
                .text();
            let epss = row
                .find(Attr("data-tsvfield", "epssScore").descendant(Name("span")))
                .nth(0)
                .unwrap()
                .text();
            let publish_date = row
                .find(Attr("data-tsvfield", "publishDate"))
                .nth(0)
                .unwrap()
                .text();
            let update_date = row
                .find(Attr("data-tsvfield", "updateDate"))
                .nth(0)
                .unwrap()
                .text();

            items.push(Cve::new(
                name,
                url,
                max_cvss,
                epss,
                publish_date,
                update_date,
            ));
        }
        let next_links = match document
            .find(Name("a").and(Attr("title", "Next page")))
            .nth(0)
            .and_then(|link| link.attr("href"))
        {
            Some(links) => vec![normalise_link(links)],
            None => vec![],
        };

        Ok((items, next_links))
    }

    async fn process(&self, item: Self::Item) -> Result<(), Error> {
        todo!()
    }
}
