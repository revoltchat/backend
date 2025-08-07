use std::collections::HashMap;

use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SearchResponse {
    pub results: Vec<TenorResult>,
    pub next: String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MediaObject {
    pub url: String,
    pub dims: Vec<u64>,
    pub duration: f64,
    pub size: f64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TenorResult {
    pub created: f64,
    #[serde(default)]
    pub hasaudio: bool,
    pub id: String,
    pub media_formats: HashMap<String, MediaObject>,
    pub tags: Vec<String>,
    pub title: String,
    pub content_description: String,
    pub itemurl: String,
    #[serde(default)]
    pub hascaption: bool,
    pub flags: Vec<String>,
    pub bg_color: Option<String>,
    pub url: String
}