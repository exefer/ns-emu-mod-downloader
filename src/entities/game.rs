#![allow(dead_code)]
use std::path::PathBuf;

#[derive(Debug)]
pub struct Game {
    pub title_id: String,
    pub title_name: String,
    pub title_version: Option<String>,
    pub mod_data_location: PathBuf,
    pub mod_download_entries: Vec<ModDownloadEntry>,
}

#[derive(Debug)]
pub struct ModDownloadEntry {
    pub download_url: String,
    pub mod_relative_path: String,
}
