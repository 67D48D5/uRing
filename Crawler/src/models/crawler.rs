// src/models/crawler.rs

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;

/// A campus containing colleges and/or departments
#[derive(Deserialize, Debug, Clone)]
pub struct Campus {
    pub campus: String,
    #[serde(default)]
    pub colleges: Vec<College>,
    #[serde(default)]
    pub departments: Vec<Department>,
}

impl Campus {
    /// Load campus configurations from a JSON file
    pub fn load_all(path: impl AsRef<Path>) -> Result<Vec<Self>> {
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    /// Get all departments with their college context
    pub fn all_departments(&self) -> Vec<DepartmentRef<'_>> {
        let mut result = Vec::new();

        for college in &self.colleges {
            for dept in &college.departments {
                result.push(DepartmentRef {
                    campus: &self.campus,
                    college: Some(&college.name),
                    dept,
                });
            }
        }

        for dept in &self.departments {
            result.push(DepartmentRef {
                campus: &self.campus,
                college: None,
                dept,
            });
        }

        result
    }
}

/// Reference to a department with context
#[derive(Debug, Clone, Copy)]
pub struct DepartmentRef<'a> {
    pub campus: &'a str,
    pub college: Option<&'a str>,
    pub dept: &'a Department,
}

/// A college containing multiple departments
#[derive(Deserialize, Debug, Clone)]
pub struct College {
    pub name: String,
    pub departments: Vec<Department>,
}

/// A department with multiple notice boards
#[derive(Deserialize, Debug, Clone)]
pub struct Department {
    pub id: String,
    pub name: String,
    pub boards: Vec<Board>,
}

/// Configuration for a single notice board
#[derive(Deserialize, Debug, Clone)]
pub struct Board {
    pub id: String,
    pub name: String,
    pub url: String,
    pub row_selector: String,
    pub title_selector: String,
    pub date_selector: String,
    #[serde(default = "Board::default_attr")]
    pub attr_name: String,
    #[serde(default)]
    pub link_selector: Option<String>,
}

impl Board {
    fn default_attr() -> String {
        "href".into()
    }
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

impl Notice {
    /// Format notice for display using a template
    pub fn format(&self, template: &str) -> String {
        template
            .replace("{dept_name}", &self.department_name)
            .replace("{board_name}", &self.board_name)
            .replace("{title}", &self.title)
            .replace("{date}", &self.date)
            .replace("{link}", &self.link)
    }
}
