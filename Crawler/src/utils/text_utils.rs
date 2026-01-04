// src/utils/text_utils.rs

use crate::models::config::CleaningConfig;

/// Apply cleaning patterns to title text
pub fn clean_title(s: &str, config: &CleaningConfig) -> String {
    let mut result = normalize_whitespace(s);
    for pattern in &config.title_remove_patterns {
        result = result.replace(pattern, "");
    }
    result.trim().to_string()
}

/// Apply cleaning patterns to date text
pub fn clean_date(s: &str, config: &CleaningConfig) -> String {
    let mut result = normalize_whitespace(s);
    for pattern in &config.date_remove_patterns {
        result = result.replace(pattern, "");
    }
    for replacement in &config.date_replacements {
        result = result.replace(&replacement.from, &replacement.to);
    }
    result.trim().to_string()
}

/// Normalize whitespace: collapse multiple spaces/newlines into single space
pub fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Format a notice for console output
pub fn format_notice(
    format: &str,
    dept_name: &str,
    board_name: &str,
    title: &str,
    date: &str,
    link: &str,
) -> String {
    format
        .replace("{dept_name}", dept_name)
        .replace("{board_name}", board_name)
        .replace("{title}", title)
        .replace("{date}", date)
        .replace("{link}", link)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::config::Replacement;

    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(normalize_whitespace("  hello   world  "), "hello world");
        assert_eq!(normalize_whitespace("hello\nworld"), "hello world");
        assert_eq!(normalize_whitespace("hello\tworld"), "hello world");
    }

    #[test]
    fn test_clean_title() {
        let config = CleaningConfig {
            title_remove_patterns: vec!["[공지]".to_string(), "[필독]".to_string()],
            date_remove_patterns: vec![],
            date_replacements: vec![],
        };
        assert_eq!(
            clean_title("  [공지] [필독] important notice  ", &config),
            "important notice"
        );
    }

    #[test]
    fn test_clean_date() {
        let config = CleaningConfig {
            title_remove_patterns: vec![],
            date_remove_patterns: vec!["작성일".to_string()],
            date_replacements: vec![Replacement {
                from: ". ".to_string(),
                to: ".".to_string(),
            }],
        };
        assert_eq!(
            clean_date("  작성일 : 2023. 01. 01.   ", &config),
            ": 2023.01.01."
        );
    }

    #[test]
    fn test_format_notice() {
        let format = "D:{dept_name}, B:{board_name}, T:{title}, D:{date}, L:{link}";
        let formatted = format_notice(
            format,
            "cs",
            "general",
            "hello",
            "2023-01-01",
            "http://example.com",
        );
        assert_eq!(
            formatted,
            "D:cs, B:general, T:hello, D:2023-01-01, L:http://example.com"
        );
    }
}
