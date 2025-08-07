use std::{sync::Arc, time::Duration};

use reqwest::Client;
use tokio::sync::RwLock;
use lru_time_cache::LruCache;
use urlencoding::encode as url_encode;
use revolt_coalesced::{CoalescionService, CoalescionServiceConfig};

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
    pub cache: Arc<RwLock<LruCache<String, Arc<types::SearchResponse>>>>
}

impl Tenor {
    pub fn new(key: &str) -> Self {
        // 1 hour, 1k queries
        let cache = LruCache::with_expiry_duration_and_capacity(Duration::from_secs(60*60), 1000);

        Self {
            key: Arc::from(key),
            client: Client::new(),
            coalescion: CoalescionService::from_config(CoalescionServiceConfig {
                max_concurrent: Some(100),
                queue_requests: true,
                max_queue: None
            }),
            cache: Arc::new(RwLock::new(cache)),
        }
    }

    pub async fn search(&self, query: &str, locale: &str, position: Option<&str>) -> Result<Arc<types::SearchResponse>, TenorError> {
        let unique_key = format!("{query}:{locale}:{position:?}");

        if self.cache.read().await.contains_key(&unique_key) {
            if let Some(response) = self.cache.write().await.get(&unique_key) {
                return Ok(response.clone())
            }
        }

        let res = self.coalescion.execute(unique_key.clone(), || {
            let client = self.client.clone();

            async move {
                let mut url = format!(
                    "{TENOR_API_BASE_URL}/search?key={}&q={}&client_key=Gifbox&locale={}&contentfilter=medium&limit=1",
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

#[cfg(test)]
mod tests {
    use super::*;
    use revolt_config::config;

    #[tokio::test(flavor = "current_thread")]
    async fn test() {
        let config = config().await;

        let tenor = Tenor::new(&config.api.security.tenor_key);

        let results =tenor.search("amog", "en_US", None).await.unwrap();

        let result = &results.results[0];

        println!("{:?}", result.media_formats.iter().next().unwrap());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_2() {
        let config = config().await;
        let tenor = Tenor::new(&config.api.security.tenor_key);

        let mut tasks = Vec::new();

        for i in 1..1001 {
            let tenor = tenor.clone();
            println!("creating search task {i}");

            tasks.push((i, tokio::spawn(async move {
                tenor.search(&format!("amog-{i}"), "en_US", None).await
            })));
        };

        for (i, task) in tasks {
            task.await.unwrap().unwrap();
            println!("Got result for {i}");
        };
    }
}