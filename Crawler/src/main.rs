// src/main.rs

mod error;
mod models;
mod services;
mod utils;

use clap::Parser;

use std::sync::Arc;

use crate::error::Result;
use crate::models::config::{Config, LocaleConfig};
use crate::models::crawler::{Campus, Notice};
use crate::services::crawling::Crawler;
use crate::utils::save_notices;

#[derive(Parser, Debug)]
#[command(version = "0.1.0", about = "A web crawler for university notices.")]
struct Args {
    #[arg(short, long, default_value = "data/config.toml")]
    config: String,

    #[arg(long, default_value = "data/locale.toml")]
    locale: String,

    #[arg(long, help = "Override site map path")]
    site_map: Option<String>,

    #[arg(short, long, help = "Override output path")]
    output: Option<String>,

    #[arg(short, long, action = clap::ArgAction::SetTrue, help = "Suppress console output")]
    quiet: bool,
}

fn print_notices(notices: &[Notice], config: &Config, locale: &LocaleConfig) {
    if !config.output.console_enabled {
        return;
    }

    println!(
        "\n{}",
        locale
            .messages
            .total_notices
            .replace("{total_count}", &notices.len().to_string())
    );
    println!("{:=<80}", locale.messages.separator_line);

    for notice in notices {
        println!("{}", notice.format(&config.output.notice_format));
        println!("{:-<80}", locale.messages.separator_short);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let locale = LocaleConfig::load_or_default(&args.locale);
    let mut config = Config::load_or_default(&args.config);

    // Apply CLI overrides
    if let Some(ref path) = args.site_map {
        config.paths.site_map = path.clone();
    }
    if let Some(ref path) = args.output {
        config.paths.output = path.clone();
    }
    if args.quiet {
        config.output.console_enabled = false;
        config.logging.show_progress = false;
    }

    if config.logging.show_progress {
        print!("{}", locale.messages.crawler_starting);
    }

    let campuses = Campus::load_all(&config.paths.site_map)?;

    let (total_depts, total_boards): (usize, usize) = campuses.iter().fold((0, 0), |(d, b), c| {
        let deps = c.all_departments();
        (
            d + deps.len(),
            b + deps.iter().map(|r| r.dept.boards.len()).sum::<usize>(),
        )
    });

    if config.logging.show_progress {
        println!(
            "{}",
            locale
                .messages
                .loaded_departments
                .replace("{count_dept}", &total_depts.to_string())
                .replace("{count_board}", &total_boards.to_string())
        );
    }

    let crawler = Crawler::new(Arc::new(config.clone()));
    let notices = crawler.fetch_all(&campuses).await?;

    print_notices(&notices, &config, &locale);
    save_notices(&notices, &config, &locale)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing() {
        let args = Args::parse_from(["crawler", "--quiet"]);
        assert!(args.quiet);
    }
}
