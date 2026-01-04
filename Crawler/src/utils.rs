// src/utils.rs

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::Result;
use crate::models::config::{Config, LocaleConfig};
use crate::models::crawler::Notice;

/// Resolve a potentially relative URL against a base URL
pub fn resolve_url(base: &url::Url, href: &str) -> String {
    base.join(href)
        .map(|u| u.to_string())
        .unwrap_or_else(|_| href.to_string())
}

/// Save notices to JSON files, organized by campus/department/board
pub fn save_notices(notices: &[Notice], config: &Config, locale: &LocaleConfig) -> Result<()> {
    if !config.output.json_enabled {
        return Ok(());
    }

    fs::create_dir_all(&config.paths.output)?;

    // Group notices: campus -> department -> board -> notices
    let mut grouped: HashMap<&str, HashMap<&str, HashMap<&str, Vec<&Notice>>>> = HashMap::new();

    for notice in notices {
        grouped
            .entry(&notice.campus)
            .or_default()
            .entry(&notice.department_name)
            .or_default()
            .entry(&notice.board_name)
            .or_default()
            .push(notice);
    }

    let output_path = Path::new(&config.paths.output);

    for (campus, departments) in grouped {
        let campus_dir = output_path.join(campus);
        fs::create_dir_all(&campus_dir)?;

        for (dept, boards) in departments {
            let dept_dir = campus_dir.join(dept);
            fs::create_dir_all(&dept_dir)?;

            for (board, board_notices) in boards {
                let safe_name = board.replace(|c: char| !c.is_alphanumeric(), "-");
                let file_path = dept_dir.join(format!("{safe_name}.json"));

                let json = if config.output.json_pretty {
                    serde_json::to_string_pretty(&board_notices)?
                } else {
                    serde_json::to_string(&board_notices)?
                };

                fs::write(&file_path, json)?;
            }
        }
    }

    if config.logging.show_progress {
        println!(
            "{}",
            locale
                .messages
                .saved_notices
                .replace("{output_path}", &config.paths.output)
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_url() {
        let base = url::Url::parse("https://example.com/path/").unwrap();
        assert_eq!(
            resolve_url(&base, "page.html"),
            "https://example.com/path/page.html"
        );
        assert_eq!(
            resolve_url(&base, "/root.html"),
            "https://example.com/root.html"
        );
        assert_eq!(
            resolve_url(&base, "https://other.com/x"),
            "https://other.com/x"
        );
    }
}
