//! URL manipulation utilities.

/// Resolve a potentially relative URL against a base URL.
///
/// # Examples
/// ```
/// use crawler::utils::url::resolve;
///
/// assert_eq!(
///     resolve("https://example.com/path/", "page.html"),
///     "https://example.com/path/page.html"
/// );
/// ```
pub fn resolve(base: &str, href: &str) -> String {
    // Already absolute
    if href.starts_with("http://") || href.starts_with("https://") {
        return href.to_string();
    }

    // Absolute path - combine with base domain
    if href.starts_with('/') {
        return resolve_absolute_path(base, href);
    }

    // Relative path - combine with base directory
    resolve_relative_path(base, href)
}

fn resolve_absolute_path(base: &str, href: &str) -> String {
    if let Some(scheme_end) = base.find("://") {
        let after_scheme = &base[scheme_end + 3..];
        if let Some(slash_idx) = after_scheme.find('/') {
            let domain = &base[..scheme_end + 3 + slash_idx];
            return format!("{domain}{href}");
        }
    }
    format!("{}{}", base.trim_end_matches('/'), href)
}

fn resolve_relative_path(base: &str, href: &str) -> String {
    let base_dir = if base.ends_with('/') {
        base.to_string()
    } else {
        match base.rfind('/') {
            Some(idx) => base[..=idx].to_string(),
            None => format!("{base}/"),
        }
    };

    format!("{base_dir}{href}")
}

/// Extract domain from a URL.
///
/// # Examples
/// ```
/// use crawler::utils::url::get_domain;
///
/// assert_eq!(
///     get_domain("https://example.com/path"),
///     Some("example.com".to_string())
/// );
/// ```
pub fn get_domain(url: &str) -> Option<String> {
    let scheme_end = url.find("://")?;
    let after_scheme = &url[scheme_end + 3..];
    let domain = after_scheme.split('/').next()?;
    Some(domain.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_absolute_url() {
        assert_eq!(
            resolve("https://example.com/path/", "https://other.com/page"),
            "https://other.com/page"
        );
    }

    #[test]
    fn test_resolve_absolute_path() {
        assert_eq!(
            resolve("https://example.com/path/", "/root.html"),
            "https://example.com/root.html"
        );
    }

    #[test]
    fn test_resolve_relative_path() {
        assert_eq!(
            resolve("https://example.com/path/", "page.html"),
            "https://example.com/path/page.html"
        );
    }

    #[test]
    fn test_resolve_relative_from_file() {
        assert_eq!(
            resolve("https://example.com/path/index.html", "other.html"),
            "https://example.com/path/other.html"
        );
    }

    #[test]
    fn test_get_domain() {
        assert_eq!(
            get_domain("https://Example.COM/path"),
            Some("example.com".to_string())
        );
        assert_eq!(get_domain("invalid-url"), None);
    }
}
