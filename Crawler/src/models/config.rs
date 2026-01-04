// src/models/config.rs

use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::error::Result;

/// Root configuration structure
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub crawler: CrawlerConfig,
    pub paths: PathsConfig,
    pub cleaning: CleaningConfig,
    pub output: OutputConfig,
    pub logging: LoggingConfig,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn load_or_default(path: impl AsRef<Path>) -> Self {
        Self::load(path).unwrap_or_else(|e| {
            eprintln!("⚠️  Config load failed: {e}. Using defaults.");
            Self::default()
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            crawler: CrawlerConfig::default(),
            paths: PathsConfig::default(),
            cleaning: CleaningConfig::default(),
            output: OutputConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

/// Locale configuration structure
#[derive(Debug, Deserialize, Clone)]
pub struct LocaleConfig {
    pub messages: Messages,
    #[allow(dead_code)] // Reserved for future localized error messages
    pub errors: Errors,
}

impl LocaleConfig {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn load_or_default(path: impl AsRef<Path>) -> Self {
        Self::load(path).unwrap_or_else(|e| {
            eprintln!("⚠️  Locale load failed: {e}. Using defaults.");
            Self::default()
        })
    }
}

impl Default for LocaleConfig {
    fn default() -> Self {
        Self {
            messages: Messages::default(),
            errors: Errors::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Messages {
    #[serde(default = "Messages::default_starting")]
    pub crawler_starting: String,
    #[serde(default = "Messages::default_loaded")]
    pub loaded_departments: String,
    #[serde(default = "Messages::default_total")]
    pub total_notices: String,
    #[serde(default = "Messages::default_saved")]
    pub saved_notices: String,
    #[serde(default = "Messages::default_sep")]
    pub separator_line: String,
    #[serde(default = "Messages::default_sep_short")]
    pub separator_short: String,
}

impl Messages {
    fn default_starting() -> String {
        "🕷️  uRing Crawler starting...\n".into()
    }
    fn default_loaded() -> String {
        "📋 Loaded {count_dept} department(s) with {count_board} board(s)\n".into()
    }
    fn default_total() -> String {
        "\n📰 Total notices fetched: {total_count}\n".into()
    }
    fn default_saved() -> String {
        "\n💾 Saved notices to {output_path}".into()
    }
    fn default_sep() -> String {
        "=".into()
    }
    fn default_sep_short() -> String {
        "-".into()
    }
}

impl Default for Messages {
    fn default() -> Self {
        Self {
            crawler_starting: Self::default_starting(),
            loaded_departments: Self::default_loaded(),
            total_notices: Self::default_total(),
            saved_notices: Self::default_saved(),
            separator_line: Self::default_sep(),
            separator_short: Self::default_sep_short(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)] // Reserved for future localized error messages
pub struct Errors {
    #[serde(default = "Errors::default_config")]
    pub config_load_failed: String,
}

impl Errors {
    fn default_config() -> String {
        "⚠️  Failed to load config: {}. Using defaults.".into()
    }
}

impl Default for Errors {
    fn default() -> Self {
        Self {
            config_load_failed: Self::default_config(),
        }
    }
}

/// Crawler behavior settings
#[derive(Debug, Deserialize, Clone)]
pub struct CrawlerConfig {
    #[serde(default = "CrawlerConfig::default_ua")]
    pub user_agent: String,
    #[serde(default = "CrawlerConfig::default_timeout")]
    pub timeout_secs: u64,
    #[serde(default = "CrawlerConfig::default_delay")]
    pub request_delay_ms: u64,
    #[serde(default = "CrawlerConfig::default_concurrent")]
    pub max_concurrent: usize,
}

impl CrawlerConfig {
    fn default_ua() -> String {
        "Mozilla/5.0 (compatible; uRing Crawler/0.1)".into()
    }
    fn default_timeout() -> u64 {
        30
    }
    fn default_delay() -> u64 {
        100
    }
    fn default_concurrent() -> usize {
        5
    }
}

impl Default for CrawlerConfig {
    fn default() -> Self {
        Self {
            user_agent: Self::default_ua(),
            timeout_secs: Self::default_timeout(),
            request_delay_ms: Self::default_delay(),
            max_concurrent: Self::default_concurrent(),
        }
    }
}

/// File path configurations
#[derive(Debug, Deserialize, Clone)]
pub struct PathsConfig {
    #[serde(default = "PathsConfig::default_sitemap")]
    pub site_map: String,
    #[serde(default = "PathsConfig::default_output")]
    pub output: String,
}

impl PathsConfig {
    fn default_sitemap() -> String {
        "data/siteMap.json".into()
    }
    fn default_output() -> String {
        "data/output".into()
    }
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            site_map: Self::default_sitemap(),
            output: Self::default_output(),
        }
    }
}

/// Text cleaning configurations
#[derive(Debug, Deserialize, Clone, Default)]
pub struct CleaningConfig {
    #[serde(default)]
    pub title_remove_patterns: Vec<String>,
    #[serde(default)]
    pub date_remove_patterns: Vec<String>,
    #[serde(default)]
    pub date_replacements: Vec<Replacement>,
}

impl CleaningConfig {
    /// Apply cleaning patterns to text
    pub fn clean(&self, s: &str, patterns: &[String], replacements: &[Replacement]) -> String {
        let mut result = Self::normalize_whitespace(s);
        for pattern in patterns {
            result = result.replace(pattern, "");
        }
        for r in replacements {
            result = result.replace(&r.from, &r.to);
        }
        result.trim().to_string()
    }

    pub fn clean_title(&self, s: &str) -> String {
        self.clean(s, &self.title_remove_patterns, &[])
    }

    pub fn clean_date(&self, s: &str) -> String {
        self.clean(s, &self.date_remove_patterns, &self.date_replacements)
    }

    fn normalize_whitespace(s: &str) -> String {
        s.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

/// A text replacement rule
#[derive(Debug, Deserialize, Clone)]
pub struct Replacement {
    pub from: String,
    pub to: String,
}

/// Output format configurations
#[derive(Debug, Deserialize, Clone)]
pub struct OutputConfig {
    #[serde(default)]
    pub console_enabled: bool,
    #[serde(default = "OutputConfig::default_json")]
    pub json_enabled: bool,
    #[serde(default = "OutputConfig::default_pretty")]
    pub json_pretty: bool,
    #[serde(default = "OutputConfig::default_format")]
    pub notice_format: String,
}

impl OutputConfig {
    fn default_json() -> bool {
        true
    }
    fn default_pretty() -> bool {
        true
    }
    fn default_format() -> String {
        "📌 [{dept_name}:{board_name}] {title}\n   📅 {date}\n   🔗 {link}".into()
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            console_enabled: false,
            json_enabled: Self::default_json(),
            json_pretty: Self::default_pretty(),
            notice_format: Self::default_format(),
        }
    }
}

/// Logging configurations
#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    #[serde(default = "LoggingConfig::default_level")]
    #[allow(dead_code)] // Reserved for future log filtering
    pub level: String,
    #[serde(default = "LoggingConfig::default_progress")]
    pub show_progress: bool,
}

impl LoggingConfig {
    fn default_level() -> String {
        "info".into()
    }
    fn default_progress() -> bool {
        true
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: Self::default_level(),
            show_progress: Self::default_progress(),
        }
    }
}
