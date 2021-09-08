use crate::util::{
    result::{Error, Result},
    variables::JANUARY_URL,
};
use linkify::{LinkFinder, LinkKind};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ImageSize {
    Large,
    Preview,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Image {
    pub url: String,
    pub width: isize,
    pub height: isize,
    pub size: ImageSize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Video {
    pub url: String,
    pub width: isize,
    pub height: isize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TwitchType {
    Channel,
    Video,
    Clip,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BandcampType {
    Album,
    Track,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Special {
    None,
    YouTube {
        id: String,
    },
    Twitch {
        content_type: TwitchType,
        id: String,
    },
    Spotify {
        content_type: String,
        id: String,
    },
    Soundcloud,
    Bandcamp {
        content_type: BandcampType,
        id: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    special: Option<Special>,

    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<Image>,
    #[serde(skip_serializing_if = "Option::is_none")]
    video: Option<Video>,

    // #[serde(skip_serializing_if = "Option::is_none")]
    // opengraph_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    site_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    colour: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Embed {
    Website(Metadata),
    Image(Image),
    None,
}

impl Embed {
    pub async fn generate(content: String) -> Result<Vec<Embed>> {
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
            .split("\n")
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
        let links: Vec<_> = finder.links(&content).take(5).collect();

        if links.len() == 0 {
            return Err(Error::LabelMe);
        }

        let mut embeds: Vec<Embed> = Vec::new();

        // ! FIXME: allow configuration of number of embeds
        // ! FIXME: batch request to january?
        let mut link_index = 0;

        while link_index < links.len() {
            let link = &links[link_index];

            // Check if we did the same link already in for this message.
            if link_index != 0 && links.iter().take(link_index).any(|x| x.as_str() == link.as_str()) {
                link_index = link_index + 1;
                continue;
            }

            let client = reqwest::Client::new();
            let result = client
                .get(&format!("{}/embed", *JANUARY_URL))
                .query(&[("url", link.as_str())])
                .send()
                .await;

            if result.is_err() {
                link_index = link_index + 1;
                continue;
            }

            let response = result.unwrap();
            if response.status().is_success() {
                let res: Embed = response.json().await.map_err(|_| Error::InvalidOperation)?;

                embeds.push(res);
            }

            link_index = link_index + 1;
        }

        // Prevent database update when no embeds are found
        if embeds.len() > 0 {
            Ok(embeds)
        } else {
            Err(Error::LabelMe)
        }
    }
}
