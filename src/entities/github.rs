#![allow(dead_code)]
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GitTree {
    pub sha: String,
    pub url: String,
    pub tree: Vec<GitTreeEntry>,
    pub truncated: bool,
}

#[derive(Debug, Deserialize)]
pub struct GitTreeEntry {
    pub path: String,
    pub mode: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub sha: String,
    pub url: String,
    pub size: Option<i64>,
}
