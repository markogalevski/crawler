mod crawler;
mod error;
mod spiders;
use env_logger;
use spiders::{CveDetailsSpider, Spider};

#[tokio::main]
async fn main() {
    unsafe { std::env::set_var("RUST_LOG", "info,crawler=debug") };
    env_logger::init();
    let spider = CveDetailsSpider::new();
    spider
        .scrape("https://www.cvedetails.com/vulnerability-list/".to_owned())
        .await
        .ok();
}
