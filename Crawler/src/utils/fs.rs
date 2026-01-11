//! File system utilities.

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::error::Result;

/// Save data to a JSON file with pretty printing.
pub fn save_json<T: serde::Serialize>(path: &Path, data: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(data)?;
    write(path, json)?;
    Ok(())
}

/// Load and parse TOML configuration from a file.
pub fn load_toml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let content = fs::read_to_string(path)?;
    let data: T = toml::from_str(&content)?;
    Ok(data)
}

/// Ensure a directory exists, creating it if necessary.
pub fn ensure_dir(path: &Path) -> Result<()> {
    create_dir_all(path)
}

/// Create all directories in a path.
pub fn create_dir_all(path: impl AsRef<Path>) -> Result<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

/// Write content to a file.
pub fn write(path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> Result<()> {
    let path = path.as_ref();

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }

    let mut file = fs::File::create(path)?;
    file.write_all(content.as_ref())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::tempdir;

    #[test]
    fn test_write_and_read() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        write(&file_path, "hello").unwrap();

        let mut content = String::new();
        fs::File::open(&file_path)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        assert_eq!(content, "hello");
    }
}
