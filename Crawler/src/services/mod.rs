// src/services/mod.rs

//! Service layer for the crawler application.
//!
//! This module contains the business logic for:
//! - Board discovery (`BoardDiscoveryService`)
//! - Department crawling (`DepartmentCrawler`)
//! - Notice fetching (`NoticeCrawler`)
//! - CMS selector detection (`SelectorDetector`)

mod boards;
mod departments;
mod notices;
mod selectors;

pub use boards::BoardDiscoveryService;
pub use departments::DepartmentCrawler;
pub use notices::NoticeCrawler;
pub use selectors::SelectorDetector;
