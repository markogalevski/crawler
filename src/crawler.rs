use futures::StreamExt;
use log;
use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread::sleep,
    time::Duration,
};
use tokio::{
    self,
    sync::{
        mpsc::{Receiver, Sender},
        Barrier,
    },
};

use crate::spiders::Spider;

type Url = String;

pub struct Crawler {
    crawling_concurrency: usize,
    processing_concurrency: usize,
    delay: Duration,
}

impl Crawler {
    pub fn new(
        delay: Duration,
        crawling_concurrency: usize,
        processing_concurrency: usize,
    ) -> Self {
        Self {
            crawling_concurrency,
            processing_concurrency,
            delay,
        }
    }
    pub async fn run<T: Send + 'static>(&self, spider: Arc<Box<dyn Spider<Item = T>>>) {
        let mut visited_urls = HashSet::<Url>::new();
        let crawling_queue_capacity = self.crawling_concurrency * 400;
        let processing_queue_capacity = self.processing_concurrency * 10;
        let num_active_spiders = Arc::new(AtomicUsize::new(0));

        let (urls_to_visit_tx, urls_to_visit_rx) =
            tokio::sync::mpsc::channel::<Url>(crawling_queue_capacity);
        let (items_tx, items_rx) = tokio::sync::mpsc::channel(processing_queue_capacity);
        let (new_urls_tx, mut new_urls_rx) = tokio::sync::mpsc::channel(crawling_queue_capacity);
        let barrier = Arc::new(Barrier::new(3)); // One barrier for each channel created above

        for url in spider.start_urls() {
            visited_urls.insert(url.clone());
            urls_to_visit_tx.send(url).await.ok();
        }

        self.launch_processors(spider.clone(), items_rx, barrier.clone())
            .await;

        self.launch_scrapers(
            spider.clone(),
            urls_to_visit_rx,
            new_urls_tx.clone(),
            items_tx,
            num_active_spiders.clone(),
            barrier.clone(),
        )
        .await;

        loop {
            if let Some(new_urls) = new_urls_rx.try_recv().ok() {
                for url in new_urls {
                    if !visited_urls.contains(&url) {
                        visited_urls.insert(url.clone());
                        log::debug!("Queueing: {url}");
                        urls_to_visit_tx.send(url).await.ok();
                    }
                }
            }

            if new_urls_tx.capacity() == new_urls_tx.max_capacity()
                && urls_to_visit_tx.capacity() == urls_to_visit_tx.max_capacity()
                && num_active_spiders.load(Ordering::SeqCst) == 0
            {
                break;
            }
            sleep(Duration::from_millis(5));
        }
        log::info!("crawler: loop exited");
        drop(urls_to_visit_tx);
        barrier.wait().await;
    }

    async fn launch_processors<T: Send + 'static>(
        &self,
        spider: Arc<Box<dyn Spider<Item = T>>>,
        items_rx: tokio::sync::mpsc::Receiver<T>,
        barrier: Arc<Barrier>,
    ) {
        let concurrency = self.processing_concurrency;
        tokio::spawn(async move {
            tokio_stream::wrappers::ReceiverStream::new(items_rx)
                .for_each_concurrent(concurrency, |item| async {
                    spider.process(item).await;
                })
                .await;
            barrier.wait().await;
        });
    }

    async fn launch_scrapers<T: Send + 'static>(
        &self,
        spider: Arc<Box<dyn Spider<Item = T>>>,
        urls_to_visit_rx: Receiver<Url>,
        new_urls_tx: Sender<Vec<Url>>,
        items_tx: Sender<T>,
        num_active_spiders: Arc<AtomicUsize>,
        barrier: Arc<Barrier>,
    ) {
        let concurrency = self.processing_concurrency;
        let delay = self.delay;
        tokio::spawn(async move {
            tokio_stream::wrappers::ReceiverStream::new(urls_to_visit_rx)
                .for_each_concurrent(concurrency, |queued_url| {
                    let queued_url = queued_url.clone();
                    async {
                        num_active_spiders.fetch_add(1, Ordering::SeqCst);
                        let mut urls = vec![];
                        let res = spider
                            .scrape(queued_url)
                            .await
                            .inspect_err(|e| log::error!("{e}"))
                            .ok();
                        if let Some((items, new_urls)) = res {
                            for item in items {
                                items_tx.send(item).await.ok();
                            }
                            urls = new_urls;
                        }
                        new_urls_tx.send(urls).await.ok();
                        sleep(delay);
                        num_active_spiders.fetch_sub(1, Ordering::SeqCst);
                    }
                })
                .await;
            drop(items_tx);
            barrier.wait().await;
        });
    }
}
