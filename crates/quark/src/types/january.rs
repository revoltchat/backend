use linkify::{LinkFinder, LinkKind};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::{models::attachment::File, Error, Result};

/// Image positioning and size
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub enum ImageSize {
    /// Show large preview at the bottom of the embed
    Large,
    /// Show small preview to the side of the embed
    Preview,
}

/// Image
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct Image {
    /// URL to the original image
    pub url: String,
    /// Width of the image
    pub width: isize,
    /// Height of the image
    pub height: isize,
    /// Positioning and size
    pub size: ImageSize,
}

/// Video
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct Video {
    /// URL to the original video
    pub url: String,
    /// Width of the video
    pub width: isize,
    /// Height of the video
    pub height: isize,
}

/// Type of remote Twitch content
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub enum TwitchType {
    Channel,
    Video,
    Clip,
}

/// Type of remote Lightspeed.tv content
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub enum LightspeedType {
    Channel,
}

/// Type of remote Bandcamp content
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub enum BandcampType {
    Album,
    Track,
}

/// Information about special remote content
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(tag = "type")]
pub enum Special {
    /// No remote content
    None,
    /// Content hint that this contains a GIF
    ///
    /// Use metadata to find video or image to play
    GIF,
    /// YouTube video
    YouTube {
        id: String,

        #[serde(skip_serializing_if = "Option::is_none")]
        timestamp: Option<String>,
    },
    /// Lightspeed.tv stream
    Lightspeed {
        content_type: LightspeedType,
        id: String,
    },
    /// Twitch stream or clip
    Twitch {
        content_type: TwitchType,
        id: String,
    },
    /// Spotify track
    Spotify { content_type: String, id: String },
    /// Soundcloud track
    Soundcloud,
    /// Bandcamp track
    Bandcamp {
        content_type: BandcampType,
        id: String,
    },
}

/// Website metadata
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct Metadata {
    /// Direct URL to web page
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    /// Original direct URL
    #[serde(skip_serializing_if = "Option::is_none")]
    original_url: Option<String>,
    /// Remote content
    #[serde(skip_serializing_if = "Option::is_none")]
    special: Option<Special>,

    /// Title of website
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    /// Description of website
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    /// Embedded image
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<Image>,
    /// Embedded video
    #[serde(skip_serializing_if = "Option::is_none")]
    video: Option<Video>,

    // #[serde(skip_serializing_if = "Option::is_none")]
    // opengraph_type: Option<String>,
    /// Site name
    #[serde(skip_serializing_if = "Option::is_none")]
    site_name: Option<String>,
    /// URL to site icon
    #[serde(skip_serializing_if = "Option::is_none")]
    icon_url: Option<String>,
    /// CSS Colour
    #[serde(skip_serializing_if = "Option::is_none")]
    colour: Option<String>,
}

/// Text Embed
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct Text {
    /// URL to icon
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// URL for title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Title of text embed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Description of text embed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// ID of uploaded autumn file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<File>,
    /// CSS Colour
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colour: Option<String>,
}

/// Embed
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(tag = "type")]
pub enum Embed {
    Website(Metadata),
    Image(Image),
    Video(Video),
    Text(Text),
    None,
}

impl Embed {
    /// Generate embeds from given content
    pub async fn generate(content: String, host: &str, max_embeds: usize) -> Result<Vec<Embed>> {
        lazy_static! {
            static ref RE_CODE: Regex = Regex::new("```(?:.|\n)+?```|`(?:.|\n)+?`").unwrap();
            static ref RE_IGNORED: Regex = Regex::new("(<http.+>)").unwrap();
        }

        // Ignore code blocks.
        let content = RE_CODE.replace_all(&content, "");

        // Ignore all content between angle brackets starting with http.
        let content = RE_IGNORED.replace_all(&content, "");

        let content = content
            // Ignore quoted lines.
            .split('\n')
            .map(|v| {
                if let Some(c) = v.chars().next() {
                    if c == '>' {
                        return "";
                    }
                }

                v
            })
            .collect::<Vec<&str>>()
            .join("\n");

        let mut finder = LinkFinder::new();
        finder.kinds(&[LinkKind::Url]);

        // Process all links, stripping anchors and
        // only taking up to `max_embeds` of links.
        let links: Vec<String> = finder
            .links(&content)
            .map(|x| {
                x.as_str()
                    .chars()
                    .take_while(|&ch| ch != '#')
                    .collect::<String>()
            })
            .collect::<HashSet<String>>()
            .into_iter()
            .take(max_embeds)
            .collect();

        // If no links, fail out.
        if links.is_empty() {
            return Err(Error::LabelMe);
        }

        // ! FIXME: batch request to january?
        let mut embeds: Vec<Embed> = Vec::new();
        let client = reqwest::Client::new();
        for link in links {
            let result = client
                .get(&format!("{}/embed", host))
                .query(&[("url", link)])
                .send()
                .await;

            if result.is_err() {
                continue;
            }

            let response = result.unwrap();
            if response.status().is_success() {
                let res: Embed = response.json().await.map_err(|_| Error::InvalidOperation)?;
                embeds.push(res);
            }
        }

        // Prevent database update when no embeds are found.
        if !embeds.is_empty() {
            Ok(embeds)
        } else {
            Err(Error::LabelMe)
        }
    }
}
