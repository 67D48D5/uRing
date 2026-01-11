// src/utils/http.rs

//! HTTP client utilities.

use std::time::Duration;

use reqwest::blocking::Client;
use scraper::Html;

use crate::error::Result;
use crate::models::CrawlerConfig;

/// Create a configured blocking HTTP client.
pub fn create_client(config: &CrawlerConfig) -> Result<Client> {
    let client = Client::builder()
        .user_agent(&config.user_agent)
        .timeout(Duration::from_secs(config.timeout_secs))
        .build()?;
    Ok(client)
}

/// Fetch a page and parse it as HTML.
pub fn fetch_page(client: &Client, url: &str) -> Result<Html> {
    let response = client.get(url).send()?;
    let text = response.text()?;
    Ok(Html::parse_document(&text))
}

/// Fetch a page with a custom timeout.
pub fn fetch_page_with_timeout(client: &Client, url: &str, timeout_secs: u64) -> Result<Html> {
    let response = client
        .get(url)
        .timeout(Duration::from_secs(timeout_secs))
        .send()?;
    let text = response.text()?;
    Ok(Html::parse_document(&text))
}
