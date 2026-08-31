//! Fetching books out, and everything that only the store's wire format cares about

pub mod api;
pub mod data;

use crate::Book;
use api::Product;
use futures::StreamExt;
use std::time::Duration;

pub struct Scraper {
    client: reqwest::Client,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error during request processing: {0}")]
    Io(#[from] reqwest::Error),
    #[error("could not decode page {page}: {source}")]
    Json {
        page: usize,
        #[source]
        source: serde_json::Error,
    },
    #[error("page {page} failed after {attempts} attempts with status {status}")]
    Status {
        page: usize,
        attempts: u32,
        status: u16,
    },
}

impl Scraper {
    const ROOT: &str = "https://lojasebocultural.com.br";
    const USER_AGENT: &str = "sebolinha/0.1";

    const ATTEMPTS: u32 = 3;
    const MAX: usize = 1 << 8;
    const PER_PAGE: usize = 100;
    const CONCURRENCY: usize = 6;

    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent(Self::USER_AGENT)
            .timeout(Duration::from_secs(30))
            .build()
            .expect("client should not fail");
        Self { client }
    }

    pub fn books(&self) -> impl futures::Stream<Item = Result<Vec<Book>, Error>> + '_ {
        let pages = 1..=Self::pages_needed(Self::MAX);
        let mut remaining = Self::MAX;

        futures::stream::iter(pages)
            .map(move |page| self.fetch_page(page))
            .buffered(Self::CONCURRENCY)
            .map(move |page| {
                page.map(|products| {
                    let mut books: Vec<_> = products.into_iter().map(Book::from).collect();
                    books.truncate(remaining);
                    remaining -= books.len();
                    books
                })
            })
    }

    pub async fn fetch_page(&self, page: usize) -> Result<Vec<Product>, Error> {
        let url = Self::url(page);
        let mut last = 0;

        for attempt in 0..Self::ATTEMPTS {
            let response = self.client.get(&url).send().await?;
            let status = response.status().as_u16();

            if response.status().is_success() {
                let body = response.text().await?;
                return serde_json::from_str(&body).map_err(|source| Error::Json { page, source });
            }

            last = status;
            if !Self::is_retryable(status) {
                break;
            }
            tokio::time::sleep(Self::backoff(attempt)).await;
        }

        Err(Error::Status {
            page,
            attempts: Self::ATTEMPTS,
            status: last,
        })
    }

    #[inline]
    fn url(page: usize) -> String {
        format!(
            "{}/wp-json/wc/store/v1/products?per_page={}&page={}&orderby=id&order=asc",
            Self::ROOT,
            Self::PER_PAGE,
            page,
        )
    }

    #[inline(always)]
    const fn pages_needed(max: usize) -> usize {
        max.div_ceil(Self::PER_PAGE)
    }

    #[inline(always)]
    fn is_retryable(status: u16) -> bool {
        status == 429 || (500..600).contains(&status)
    }

    #[inline]
    const fn backoff(attempt: u32) -> Duration {
        Duration::from_millis(500 * 2u64.pow(attempt))
    }
}

impl Default for Scraper {
    fn default() -> Self {
        Self::new()
    }
}
