use std::{
    io::{Cursor, Write},
    time::Duration,
};

use lazy_static::lazy_static;
use mime::Mime;
use reqwest::{header::CONTENT_TYPE, redirect, Client, Response};
use revolt_config::report_internal_error;
use revolt_files::{create_thumbnail, decode_image, is_valid_image, video_size};
use revolt_models::v0::Embed;
use revolt_result::{create_error, Result};

lazy_static! {
    static ref CLIENT: Client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; January/2.0; +https://github.com/revoltchat/backend)")
        .timeout(Duration::from_secs(10)) // TODO config
        .connect_timeout(Duration::from_secs(5)) // TODO config
        .redirect(redirect::Policy::custom(|attempt| {
            if attempt.previous().len() > 5 { // TODO config
                attempt.error("too many redirects")
            } else if attempt.url().host_str() == Some("jan.revolt.chat") { // TODO config
                attempt.stop()
            } else {
                attempt.follow()
            }
        }))
        .build()
        .expect("reqwest Client");

    /// Cache for proxy results
    static ref PROXY_CACHE: moka::future::Cache<String, Result<(String, Vec<u8>)>> = moka::future::Cache::builder()
        .max_capacity(10_000) // TODO config
        .time_to_live(Duration::from_secs(60)) // TODO config
        .build();

    /// Cache for embed results
    static ref EMBED_CACHE: moka::future::Cache<String, Result<Embed>> = moka::future::Cache::builder()
        .max_capacity(1_000) // TODO config
        .time_to_live(Duration::from_secs(60)) // TODO config
        .build();
}

/// Information about a successful request
pub struct Request {
    response: Response,
    mime: Mime,
}

impl Request {
    /// Proxy a given URL
    pub async fn proxy_file(url: &str) -> Result<(String, Vec<u8>)> {
        if let Some(hit) = PROXY_CACHE.get(url).await {
            hit
        } else {
            let Request { response, mime } = Request::new(url).await?;

            if matches!(mime.type_(), mime::IMAGE | mime::VIDEO) {
                let bytes = response.bytes().await.map_err(|_| create_error!(LabelMe));

                let result = match bytes {
                    Ok(bytes) => {
                        if matches!(mime.type_(), mime::IMAGE) {
                            let reader = &mut Cursor::new(&bytes);

                            if matches!(mime.subtype(), mime::GIF) {
                                if is_valid_image(reader, false) {
                                    Ok(("image/gif".to_owned(), bytes.to_vec()))
                                } else {
                                    Err(create_error!(LabelMe))
                                }
                            } else {
                                Ok((
                                    "image/webp".to_owned(),
                                    create_thumbnail(decode_image(reader, false)?, "attachments")
                                        .await,
                                ))
                            }
                        } else {
                            let mut file = report_internal_error!(tempfile::NamedTempFile::new())?;
                            report_internal_error!(file.write_all(&bytes))?;
                            if video_size(&file).is_some() {
                                Ok((mime.to_string(), bytes.to_vec()))
                            } else {
                                Err(create_error!(LabelMe))
                            }
                        }
                    }
                    Err(err) => Err(err),
                };

                PROXY_CACHE.insert(url.to_owned(), result.clone()).await;
                result
            } else {
                // Err(create_error!())
                todo!() // no proxy
            }
        }
    }

    /// Generate embed for a given URL
    pub async fn generate_embed(url: &str) -> Result<Embed> {
        if let Some(hit) = EMBED_CACHE.get(url).await {
            hit
        } else {
            todo!()
        }
    }

    /// Send a new request to a service
    pub async fn new(url: &str) -> Result<Request> {
        let response = CLIENT
            .get(url)
            .send()
            .await
            .map_err(|_| create_error!(ProxyError))?;

        if !response.status().is_success() {
            tracing::error!("{:?}", response);
            return Err(create_error!(ProxyError));
        }

        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .ok_or(create_error!(ProxyError))?
            .to_str()
            .map_err(|_| create_error!(ProxyError))?;

        let mime: mime::Mime = content_type
            .parse()
            .map_err(|_| create_error!(ProxyError))?;

        Ok(Request { response, mime })
    }
}
