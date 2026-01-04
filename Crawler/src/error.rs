// src/error.rs

use thiserror::Error;

pub type Result<T> = std::result::Result<T, CrawlerError>;

#[derive(Error, Debug)]
pub enum CrawlerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON parsing failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parsing failed: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("URL parsing failed: {0}")]
    Url(#[from] url::ParseError),

    #[error("Invalid selector '{selector}': {message}")]
    Selector { selector: String, message: String },

    #[error("Configuration error: {0}")]
    #[allow(dead_code)] // Reserved for future use
    Config(String),
}
