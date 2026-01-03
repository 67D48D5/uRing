// src/models/crawler.rs

use serde::{Deserialize, Serialize};

/// A campus containing colleges and/or departments
/// The JSON structure can have either:
/// - "colleges" array (with nested departments)
/// - "departments" array (directly under campus)
/// - Both
#[derive(Deserialize, Debug, Clone)]
pub struct Campus {
    pub campus: String,
    #[serde(default)]
    pub colleges: Vec<College>,
    #[serde(default)]
    pub departments: Vec<Department>,
}

impl Campus {
    /// Get all departments from this campus (both from colleges and direct departments)
    pub fn all_departments(&self) -> Vec<(&str, &Department)> {
        let mut result = Vec::new();

        // Add departments from colleges (with college name context)
        for college in &self.colleges {
            for dept in &college.departments {
                result.push((college.name.as_str(), dept));
            }
        }

        // Add direct departments (no college context)
        for dept in &self.departments {
            result.push(("", dept));
        }

        result
    }
}

/// A college containing multiple departments
#[derive(Deserialize, Debug, Clone)]
pub struct College {
    pub name: String,
    pub departments: Vec<Department>,
}

/// A department/organization with multiple notice boards
#[derive(Deserialize, Debug, Clone)]
pub struct Department {
    pub id: String,
    pub name: String,
    pub boards: Vec<BoardConfig>,
}

/// Configuration for a single notice board
#[derive(Deserialize, Debug, Clone)]
pub struct BoardConfig {
    pub id: String,
    pub name: String,
    pub url: String,
    pub row_selector: String,
    pub title_selector: String,
    pub date_selector: String,
    pub attr_name: String,
    /// Optional link selector (if different from title_selector)
    #[serde(default)]
    pub link_selector: Option<String>,
}

/// A notice fetched from a board
#[derive(Debug, Clone, Serialize)]
pub struct Notice {
    pub campus: String,
    pub college: String,
    pub department_id: String,
    pub department_name: String,
    pub board_id: String,
    pub board_name: String,
    pub title: String,
    pub date: String,
    pub link: String,
}
