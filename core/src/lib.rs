pub mod config;
pub mod diagnostics;
pub mod engines;
pub mod error;
pub mod files;
pub mod glossary;
pub mod hardware;
pub mod languages;
pub mod media;
pub mod models;
pub mod paths;
pub mod pipeline;
pub mod project;
pub mod segments;
pub mod subtitles;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
