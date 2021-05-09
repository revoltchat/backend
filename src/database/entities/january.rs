use serde::{Serialize, Deserialize};
use linkify::{LinkFinder, LinkKind};
use crate::util::{result::{Error, Result}, variables::JANUARY_URL};

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
    Track
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
        id: String
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    url: Option<String>,
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
    color: Option<String>,
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
        let content = content
            .split("\n")
            .map(|v| {
                // Ignore quoted lines.
                if let Some(c) = v.chars().next() {
                    if c == '>' {
                        return ""
                    }
                }

                v
            })
            // This will also remove newlines, but
            // that isn't important here.
            .collect::<String>();

        // ! FIXME: allow multiple links
        // ! FIXME: prevent generation if link is surrounded with < >
        let mut finder = LinkFinder::new();
        finder.kinds(&[LinkKind::Url]);
        let links: Vec<_> = finder.links(&content).collect();

        if links.len() == 0 {
            return Err(Error::LabelMe);
        }

        let link = &links[0];

        let client = reqwest::Client::new();
        let result = client
            .get(&format!("{}/embed", *JANUARY_URL))
            .query(&[("url", link.as_str())])
            .send()
            .await;

        match result {
            Err(_) => return Err(Error::LabelMe),
            Ok(result) => match result.status() {
                reqwest::StatusCode::OK => {
                    let res: Embed = result.json()
                        .await
                        .map_err(|_| Error::InvalidOperation)?;

                    Ok(vec![ res ])
                },
                _ => return Err(Error::LabelMe),
            },
        }
    }
}
