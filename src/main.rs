mod crawler;
mod error;
mod spiders;
use std::{sync::Arc, time::Duration};

use clap::{self, Parser, Subcommand};
use crawler::Crawler;
use env_logger;

#[derive(Parser)]
#[command(version, about, long_about=Some("about"))]
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all spiders
    Spiders,
    /// Run a spider
    Crawl {
        #[arg(short, long)]
        spider: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), crate::error::Error> {
    unsafe { std::env::set_var("RUST_LOG", "info,crawler=debug") };
    env_logger::init();
    let cli = Cli::parse();
    match cli.commands {
        Commands::Spiders => cli::list_spiders(),
        Commands::Crawl { spider } => {
            let crawler = Crawler::new(Duration::from_millis(200), 2, 500);
            let spider = spiders::Spiders::try_from(spider.as_str())?.inner();
            crawler.run(Arc::new(Box::new(spider))).await;
        }
    }
    Ok(())
}

mod cli {
    use crate::spiders::{self, get_spider_names};
    pub fn list_spiders() {
        for name in get_spider_names() {
            println!("{name}");
        }
    }
}
