#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Repository {
    pub url: String,
    pub assets_url: String,
    pub upload_url: String,
    pub html_url: String,
    pub id: i64,
    pub author: Author,
    pub node_id: String,
    pub tag_name: String,
    pub target_commitish: String,
    pub name: String,
    pub draft: bool,
    pub immutable: bool,
    pub prerelease: bool,
    pub created_at: String,
    pub published_at: String,
    pub assets: Vec<Asset>,
    pub tarball_url: String,
    pub zipball_url: String,
    pub body: String,
    pub reactions: Reactions,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Author {
    pub login: String,
    pub id: i64,
    pub node_id: String,
    pub avatar_url: String,
    pub gravatar_id: String,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub user_view_type: String,
    pub site_admin: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Asset {
    pub url: String,
    pub id: i64,
    pub node_id: String,
    pub name: String,
    pub label: Value,
    pub uploader: Uploader,
    pub content_type: String,
    pub state: String,
    pub size: i64,
    pub digest: Value,
    pub download_count: i64,
    pub created_at: String,
    pub updated_at: String,
    pub browser_download_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Uploader {
    pub login: String,
    pub id: i64,
    pub node_id: String,
    pub avatar_url: String,
    pub gravatar_id: String,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub user_view_type: String,
    pub site_admin: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reactions {
    pub url: String,
    pub total_count: i64,
    #[serde(rename = "+1")]
    pub n1: i64,
    #[serde(rename = "-1")]
    pub n12: i64,
    pub laugh: i64,
    pub hooray: i64,
    pub confused: i64,
    pub heart: i64,
    pub rocket: i64,
    pub eyes: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitTree {
    pub sha: String,
    pub url: String,
    pub tree: Vec<GitTreeEntry>,
    pub truncated: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitTreeEntry {
    pub path: String,
    pub mode: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub sha: String,
    pub url: String,
    pub size: Option<i64>,
}
