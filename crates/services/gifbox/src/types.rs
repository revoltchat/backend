use std::collections::HashMap;

use utoipa::ToSchema;
use serde::{Serialize, Deserialize};

use crate::tenor::types;

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, PartialEq)]
/// Response containing the current results and the id of the next result for pagination.
pub struct SearchResponse {
    /// Current gif results.
    pub results: Vec<MediaResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Id of the next result.
    pub next: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, PartialEq)]
/// Indivual gif result.
pub struct MediaResult {
    /// Unique Tenor id.
    pub id: String,
    /// Mapping of each file format and url of the file.
    pub media_formats: HashMap<String, MediaObject>,
    /// Public Tenor web url for the gif.
    pub url: String
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, PartialEq)]
/// Represents the gif in a certain file format.
pub struct MediaObject {
    /// File url of the gif in a certain format.
    pub url: String,
    /// Width and height of the file in px.
    pub dimensions: Vec<u64>,
}


impl From<types::SearchResponse> for SearchResponse {
    fn from(value: types::SearchResponse) -> Self {
        Self {
            results: value.results.into_iter().map(|result| result.into()).collect(),
            next: if value.next.is_empty() {
                None
            } else {
                Some(value.next)
            }
        }
    }
}

impl From<types::MediaResult> for MediaResult {
    fn from(value: types::MediaResult) -> Self {
        Self {
            id: value.id,
            media_formats: value.media_formats.into_iter().map(|(k, v)| (k, v.into())).collect(),
            url: value.url
        }
    }
}

impl From<types::MediaObject> for MediaObject {
    fn from(value: types::MediaObject) -> Self {
        Self {
            url: value.url,
            dimensions: value.dims
        }
    }
}