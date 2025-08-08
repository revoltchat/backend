use std::{sync::Arc, time::Duration};

use lru_time_cache::LruCache;
use reqwest::Client;
use revolt_coalesced::{CoalescionService, CoalescionServiceConfig};
use tokio::sync::RwLock;
use urlencoding::encode as url_encode;

pub mod types;

const TENOR_API_BASE_URL: &str = "https://tenor.googleapis.com/v2";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TenorError {
    HttpError,
}

#[derive(Clone)]
pub struct Tenor {
    pub key: Arc<str>,
    pub client: Client,
    pub coalescion: CoalescionService<String, Result<Arc<types::SearchResponse>, TenorError>>,
    pub cache: Arc<RwLock<LruCache<String, Arc<types::SearchResponse>>>>,
}

impl Tenor {
    pub fn new(key: &str) -> Self {
        // 1 hour, 1k queries
        let cache = LruCache::with_expiry_duration_and_capacity(Duration::from_secs(60 * 60), 1000);

        Self {
            key: Arc::from(key),
            client: Client::new(),
            coalescion: CoalescionService::from_config(CoalescionServiceConfig {
                max_concurrent: Some(100),
                queue_requests: true,
                max_queue: None,
            }),
            cache: Arc::new(RwLock::new(cache)),
        }
    }

    pub async fn search(
        &self,
        query: &str,
        locale: &str,
        position: Option<&str>,
    ) -> Result<Arc<types::SearchResponse>, TenorError> {
        let unique_key = format!("{query}:{locale}:{position:?}");

        if self.cache.read().await.contains_key(&unique_key) {
            if let Some(response) = self.cache.write().await.get(&unique_key) {
                return Ok(response.clone());
            }
        }

        let res = self.coalescion.execute(unique_key.clone(), || {
            let client = self.client.clone();

            async move {
                let mut url = format!(
                    "{TENOR_API_BASE_URL}/search?key={}&q={}&client_key=Gifbox&media_filter=webm,tinywebm&locale={}&contentfilter=medium&limit=1",
                    &self.key,
                    url_encode(query),
                    url_encode(locale),
                );

                if let Some(position) = position {
                    url.push_str("&pos=");
                    url.push_str(position);
                };

                let response = client.get(url)
                    .send()
                    .await
                    .inspect_err(|e| { revolt_config::capture_error(e); })
                    .map_err(|_| TenorError::HttpError)?;

                let text = response.text().await.map_err(|e| { println!("{e:?}"); TenorError::HttpError })?;

                Ok(Arc::new(serde_json::from_str(&text).unwrap()))
            }
        })
        .await
        .unwrap();

        if let Ok(resp) = &*res {
            self.cache.write().await.insert(unique_key, resp.clone());
        }

        (*res).clone()
    }
}
