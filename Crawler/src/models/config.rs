//! Application configuration structures.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::Result;

// ============================================================================
// Main Configuration
// ============================================================================

/// Root application configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// HTTP and crawling behavior settings
    #[serde(default)]
    pub crawler: CrawlerConfig,

    /// File path settings
    #[serde(default)]
    pub paths: PathsConfig,

    /// Board discovery rules
    #[serde(default)]
    pub discovery: DiscoveryConfig,

    /// Text preprocessing settings
    #[serde(default)]
    pub cleaning: CleaningConfig,

    /// Output format settings
    #[serde(default)]
    pub output: OutputConfig,

    /// Logging settings
    #[serde(default)]
    pub logging: LoggingConfig,
}

impl Config {
    /// Load configuration from a TOML file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    /// Load configuration or return default if loading fails.
    pub fn load_or_default(path: impl AsRef<Path>) -> Self {
        Self::load(&path).unwrap_or_else(|e| {
            eprintln!(
                "⚠️  Config load failed from {:?}: {e}. Using defaults.",
                path.as_ref()
            );
            Self::default()
        })
    }

    // Path helper methods

    /// Get the full path to the output directory.
    pub fn output_dir(&self, base: &Path) -> PathBuf {
        base.join(&self.paths.output_dir)
    }

    /// Get the full path to the seed file.
    pub fn seed_path(&self, base: &Path) -> PathBuf {
        base.join(&self.paths.seed_file)
    }

    /// Get the full path to departments file.
    pub fn departments_path(&self, base: &Path) -> PathBuf {
        self.output_dir(base).join(&self.paths.departments_file)
    }

    /// Get the full path to departments with boards file.
    pub fn departments_boards_path(&self, base: &Path) -> PathBuf {
        self.output_dir(base)
            .join(&self.paths.departments_boards_file)
    }

    /// Get the full path to manual review file.
    pub fn manual_review_path(&self, base: &Path) -> PathBuf {
        self.output_dir(base).join(&self.paths.manual_review_file)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            crawler: CrawlerConfig::default(),
            paths: PathsConfig::default(),
            discovery: DiscoveryConfig::default(),
            cleaning: CleaningConfig::default(),
            output: OutputConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

// ============================================================================
// Crawler Settings
// ============================================================================

/// HTTP client and crawling behavior settings.
#[derive(Debug, Clone, Deserialize)]
pub struct CrawlerConfig {
    /// User-Agent header for HTTP requests
    #[serde(default = "defaults::user_agent")]
    pub user_agent: String,

    /// Request timeout in seconds
    #[serde(default = "defaults::timeout")]
    pub timeout_secs: u64,

    /// Longer timeout for sitemap/discovery requests
    #[serde(default = "defaults::sitemap_timeout")]
    pub sitemap_timeout_secs: u64,

    /// Delay between requests in milliseconds
    #[serde(default = "defaults::request_delay")]
    pub request_delay_ms: u64,

    /// Maximum concurrent requests
    #[serde(default = "defaults::max_concurrent")]
    pub max_concurrent: usize,
}

impl Default for CrawlerConfig {
    fn default() -> Self {
        Self {
            user_agent: defaults::user_agent(),
            timeout_secs: defaults::timeout(),
            sitemap_timeout_secs: defaults::sitemap_timeout(),
            request_delay_ms: defaults::request_delay(),
            max_concurrent: defaults::max_concurrent(),
        }
    }
}

// ============================================================================
// Path Settings
// ============================================================================

/// File path configurations.
#[derive(Debug, Clone, Deserialize)]
pub struct PathsConfig {
    /// Path to seed file (relative to project root)
    #[serde(default = "defaults::seed_file")]
    pub seed_file: String,

    /// Output directory path (relative to project root)
    #[serde(default = "defaults::output_dir")]
    pub output_dir: String,

    /// Output filename for crawled data
    #[serde(default = "defaults::output_dir")]
    pub output: String,

    /// Departments list filename
    #[serde(default = "defaults::departments_file")]
    pub departments_file: String,

    /// Departments with boards filename
    #[serde(default = "defaults::departments_boards_file")]
    pub departments_boards_file: String,

    /// Manual review items filename
    #[serde(default = "defaults::manual_review_file")]
    pub manual_review_file: String,
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            seed_file: defaults::seed_file(),
            output_dir: defaults::output_dir(),
            output: defaults::output_dir(),
            departments_file: defaults::departments_file(),
            departments_boards_file: defaults::departments_boards_file(),
            manual_review_file: defaults::manual_review_file(),
        }
    }
}

// ============================================================================
// Discovery Settings
// ============================================================================

/// Board discovery settings.
#[derive(Debug, Clone, Deserialize)]
pub struct DiscoveryConfig {
    /// Maximum length for board name (longer text is likely a notice title)
    #[serde(default = "defaults::max_board_name_length")]
    pub max_board_name_length: usize,

    /// URL patterns to exclude from board discovery
    #[serde(default = "defaults::blacklist_patterns")]
    pub blacklist_patterns: Vec<String>,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            max_board_name_length: defaults::max_board_name_length(),
            blacklist_patterns: defaults::blacklist_patterns(),
        }
    }
}

// ============================================================================
// Cleaning Settings
// ============================================================================

/// Text cleaning/preprocessing settings.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct CleaningConfig {
    /// Patterns to remove from titles
    #[serde(default)]
    pub title_remove_patterns: Vec<String>,

    /// Patterns to remove from dates
    #[serde(default)]
    pub date_remove_patterns: Vec<String>,

    /// Text replacements to apply to dates
    #[serde(default)]
    pub date_replacements: Vec<Replacement>,
}

impl CleaningConfig {
    /// Clean text by removing patterns and applying replacements.
    fn clean(&self, text: &str, patterns: &[String], replacements: &[Replacement]) -> String {
        let mut result = Self::normalize_whitespace(text);

        for pattern in patterns {
            result = result.replace(pattern, "");
        }

        for r in replacements {
            result = result.replace(&r.from, &r.to);
        }

        result.trim().to_string()
    }

    /// Clean a title string.
    pub fn clean_title(&self, text: &str) -> String {
        self.clean(text, &self.title_remove_patterns, &[])
    }

    /// Clean a date string.
    pub fn clean_date(&self, text: &str) -> String {
        self.clean(text, &self.date_remove_patterns, &self.date_replacements)
    }

    fn normalize_whitespace(s: &str) -> String {
        s.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

/// A text replacement rule.
#[derive(Debug, Clone, Deserialize)]
pub struct Replacement {
    pub from: String,
    pub to: String,
}

// ============================================================================
// Output Settings
// ============================================================================

/// Output format settings.
#[derive(Debug, Clone, Deserialize)]
pub struct OutputConfig {
    /// Enable console output
    #[serde(default)]
    pub console_enabled: bool,

    /// Enable JSON file output
    #[serde(default = "defaults::json_enabled")]
    pub json_enabled: bool,

    /// Pretty-print JSON output
    #[serde(default = "defaults::json_pretty")]
    pub json_pretty: bool,

    /// Template for notice display
    #[serde(default = "defaults::notice_format")]
    pub notice_format: String,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            console_enabled: false,
            json_enabled: defaults::json_enabled(),
            json_pretty: defaults::json_pretty(),
            notice_format: defaults::notice_format(),
        }
    }
}

// ============================================================================
// Logging Settings
// ============================================================================

/// Logging settings.
#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    /// Log level (debug, info, warn, error)
    #[serde(default = "defaults::log_level")]
    pub level: String,

    /// Show progress indicators
    #[serde(default = "defaults::show_progress")]
    pub show_progress: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: defaults::log_level(),
            show_progress: defaults::show_progress(),
        }
    }
}

// ============================================================================
// Locale Configuration
// ============================================================================

/// Internationalization/localization settings.
#[derive(Debug, Clone, Deserialize)]
pub struct LocaleConfig {
    /// UI messages
    pub messages: Messages,

    /// Error messages
    #[serde(default)]
    pub errors: Errors,
}

impl LocaleConfig {
    /// Load locale from a TOML file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    /// Load locale or return default if loading fails.
    pub fn load_or_default(path: impl AsRef<Path>) -> Self {
        Self::load(&path).unwrap_or_else(|e| {
            eprintln!(
                "⚠️  Locale load failed from {:?}: {e}. Using defaults.",
                path.as_ref()
            );
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

/// UI message strings.
#[derive(Debug, Clone, Deserialize)]
pub struct Messages {
    #[serde(default = "defaults::msg_starting")]
    pub crawler_starting: String,

    #[serde(default = "defaults::msg_loaded")]
    pub loaded_departments: String,

    #[serde(default = "defaults::msg_total")]
    pub total_notices: String,

    #[serde(default = "defaults::msg_saved")]
    pub saved_notices: String,

    #[serde(default = "defaults::msg_separator")]
    pub separator_line: String,

    #[serde(default = "defaults::msg_separator_short")]
    pub separator_short: String,
}

impl Default for Messages {
    fn default() -> Self {
        Self {
            crawler_starting: defaults::msg_starting(),
            loaded_departments: defaults::msg_loaded(),
            total_notices: defaults::msg_total(),
            saved_notices: defaults::msg_saved(),
            separator_line: defaults::msg_separator(),
            separator_short: defaults::msg_separator_short(),
        }
    }
}

/// Error message strings.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Errors {
    #[serde(default = "defaults::err_config_load")]
    pub config_load_failed: String,
}

// ============================================================================
// Default Values Module
// ============================================================================

mod defaults {
    // Crawler defaults
    pub fn user_agent() -> String {
        "Mozilla/5.0 (compatible; uRing/1.0)".into()
    }
    pub fn timeout() -> u64 {
        30
    }
    pub fn sitemap_timeout() -> u64 {
        10
    }
    pub fn request_delay() -> u64 {
        100
    }
    pub fn max_concurrent() -> usize {
        5
    }

    // Path defaults
    pub fn seed_file() -> String {
        "data/seed.toml".into()
    }
    pub fn output_dir() -> String {
        "data/output".into()
    }
    pub fn departments_file() -> String {
        "yonsei_departments.json".into()
    }
    pub fn departments_boards_file() -> String {
        "yonsei_departments_boards.json".into()
    }
    pub fn manual_review_file() -> String {
        "manual_review_needed.json".into()
    }

    // Discovery defaults
    pub fn max_board_name_length() -> usize {
        20
    }
    pub fn blacklist_patterns() -> Vec<String> {
        vec![
            "articleNo".into(),
            "article_no".into(),
            "mode=view".into(),
            "seq".into(),
            "view.do".into(),
            "board_seq".into(),
        ]
    }

    // Output defaults
    pub fn json_enabled() -> bool {
        true
    }
    pub fn json_pretty() -> bool {
        true
    }
    pub fn notice_format() -> String {
        "📌 [{dept_name}:{board_name}] {title}\n   📅 {date}\n   🔗 {link}".into()
    }

    // Logging defaults
    pub fn log_level() -> String {
        "info".into()
    }
    pub fn show_progress() -> bool {
        true
    }

    // Message defaults
    pub fn msg_starting() -> String {
        "🕷️  uRing Crawler starting...\n".into()
    }
    pub fn msg_loaded() -> String {
        "📋 Loaded {count_dept} department(s) with {count_board} board(s)\n".into()
    }
    pub fn msg_total() -> String {
        "\n📰 Total notices fetched: {total_count}\n".into()
    }
    pub fn msg_saved() -> String {
        "\n💾 Saved notices to {output_path}".into()
    }
    pub fn msg_separator() -> String {
        "=".into()
    }
    pub fn msg_separator_short() -> String {
        "-".into()
    }

    // Error defaults
    pub fn err_config_load() -> String {
        "⚠️  Failed to load config: {}. Using defaults.".into()
    }
}
