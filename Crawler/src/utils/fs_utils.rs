// src/utils/fs_utils.rs

use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

use crate::models::config::{Config, LocaleConfig};
use crate::models::crawler::{Campus, Notice};

/// Load campus configurations from a JSON file
pub fn load_campuses<P: AsRef<Path>>(path: P) -> Result<Vec<Campus>, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let campuses: Vec<Campus> = serde_json::from_str(&content)?;
    Ok(campuses)
}

/// Save notices to JSON files, organized by campus and department
pub fn save_notices_to_files(
    notices: &[Notice],
    config: &Config,
    locale: &LocaleConfig,
) -> Result<(), Box<dyn Error>> {
    if !config.output.json_enabled {
        return Ok(());
    }

    fs::create_dir_all(&config.paths.output)?;

    let mut notices_by_campus: HashMap<String, HashMap<String, HashMap<String, Vec<&Notice>>>> =
        HashMap::new();

    for notice in notices {
        notices_by_campus
            .entry(notice.campus.clone())
            .or_default()
            .entry(notice.department_name.clone())
            .or_default()
            .entry(notice.board_name.clone())
            .or_default()
            .push(notice);
    }

    for (campus_name, departments) in notices_by_campus {
        let campus_dir = Path::new(&config.paths.output).join(&campus_name);
        fs::create_dir_all(&campus_dir)?;

        for (dept_name, boards) in departments {
            let dept_dir = campus_dir.join(&dept_name);
            fs::create_dir_all(&dept_dir)?;

            for (board_name, board_notices) in boards {
                let safe_board_name = board_name.replace(|c: char| !c.is_alphanumeric(), "-");
                let file_path = dept_dir.join(format!("{}.json", safe_board_name));

                let json_output = if config.output.json_pretty {
                    serde_json::to_string_pretty(&board_notices)?
                } else {
                    serde_json::to_string(&board_notices)?
                };

                fs::write(&file_path, &json_output)?;
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
