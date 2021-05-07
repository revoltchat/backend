use serde::{Serialize, Deserialize};
use linkify::{LinkFinder, LinkKind};
use crate::util::{result::{Error, Result}, variables::JANUARY_URL};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MediaSize {
    Large,
    Preview,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Media {
    pub url: String,
    pub width: isize,
    pub height: isize,
    pub size: MediaSize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<Media>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Embed {
    Website(Metadata),
    Image(Media),
    None,
}

impl Embed {
    pub async fn generate(content: String) -> Result<Vec<Embed>> {
        // FIXME: allow multiple links
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
