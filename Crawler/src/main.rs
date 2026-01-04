// src/main.rs

mod config;
mod locale;
mod models;
mod services;
mod utils;

use clap::Parser;

use std::error::Error;
use std::sync::Arc;

use crate::locale::load_locale_or_default;
use crate::services::crawling::{Crawler, ReqwestHtmlFetcher};

use crate::models::config::{Config, LocaleConfig};
use crate::models::crawler::Notice;

use crate::utils::fs_utils::{load_campuses, save_notices_to_files};
use crate::utils::text_utils::format_notice;

// A simple struct to hold the parsed arguments
#[derive(Parser, Debug)]
#[command(version = "0.1.0", about = "A web crawler for university notices.")]
struct Args {
    #[arg(
        short,
        long,
        default_value = "data/config.toml",
        help = "Sets a custom config file"
    )]
    config: String,

    #[arg(
        long,
        default_value = "data/locale.toml",
        help = "Sets a custom locale file"
    )]
    locale: String,

    #[arg(long, help = "Overrides the site map path from the config")]
    site_map: Option<String>,

    #[arg(short, long, help = "Overrides the output path from the config")]
    output: Option<String>,

    #[arg(short, long, action = clap::ArgAction::SetTrue, help = "Suppresses console output")]
    quiet: bool,
}

fn present_notices_to_console(notices: &[Notice], config: &Config, locale: &LocaleConfig) {
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
        let formatted = format_notice(
            &config.output.notice_format,
            &notice.department_name,
            &notice.board_name,
            &notice.title,
            &notice.date,
            &notice.link,
        );
        println!("{}", formatted);
        println!("{:-<80}", locale.messages.separator_short);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let locale = load_locale_or_default(&args.locale);
    let mut config = Config::load_or_default(&args.config, &locale);

    // Apply CLI overrides
    if let Some(site_map) = args.site_map {
        config.paths.site_map = site_map;
    }
    if let Some(output) = args.output {
        config.paths.output = output;
    }
    if args.quiet {
        config.output.console_enabled = false;
        config.logging.show_progress = false;
    }

    if config.logging.show_progress {
        print!("{}", locale.messages.crawler_starting);
    }

    let campuses = load_campuses(&config.paths.site_map)?;

    let total_boards: usize = campuses
        .iter()
        .map(|c| {
            c.all_departments()
                .iter()
                .map(|(_, d)| d.boards.len())
                .sum::<usize>()
        })
        .sum();
    let total_depts: usize = campuses.iter().map(|c| c.all_departments().len()).sum();

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

    let config_arc = Arc::new(config.clone());
    let html_fetcher = Arc::new(ReqwestHtmlFetcher::new(&config));
    let crawler = Crawler::new(config_arc.clone(), html_fetcher);

    let notices = crawler.fetch_all_notices(&campuses).await?;

    present_notices_to_console(&notices, &config, &locale);
    save_notices_to_files(&notices, &config, &locale)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing() {
        let args = Args::parse_from(&["crawler", "--quiet"]);
        assert!(args.quiet);
    }
}
