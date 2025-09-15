//! Interal Tenor API wrapper

use std::{sync::Arc, time::Duration};

use lru_time_cache::LruCache;
use reqwest::Client;
use revolt_coalesced::{CoalescionService, CoalescionServiceConfig};
use serde::de::DeserializeOwned;
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
    pub coalescion: CoalescionService<String>,
    pub cache: Arc<RwLock<LruCache<String, Arc<types::PaginatedMediaResponse>>>>,

    pub categories: Arc<RwLock<LruCache<String, Arc<types::CategoriesResponse>>>>,
    pub featured: Arc<RwLock<LruCache<String, Arc<types::PaginatedMediaResponse>>>>,
}

impl Tenor {
    pub fn new(key: &str) -> Self {
        Self {
            key: Arc::from(key),
            client: Client::new(),
            coalescion: CoalescionService::from_config(CoalescionServiceConfig {
                max_concurrent: Some(100),
                queue_requests: true,
                max_queue: None,
            }),

            // 1 hour, 1k requests
            cache: Arc::new(RwLock::new(LruCache::with_expiry_duration_and_capacity(
                Duration::from_secs(60 * 60),
                1000,
            ))),

            // 1 day, 1k requests
            categories: Arc::new(RwLock::new(LruCache::with_expiry_duration_and_capacity(
                Duration::from_secs(60 * 60 * 24),
                1000,
            ))),

            // 1 day, 1k requests
            featured: Arc::new(RwLock::new(LruCache::with_expiry_duration_and_capacity(
                Duration::from_secs(60 * 60 * 24),
                1000,
            ))),
        }
    }

    pub async fn request<T: DeserializeOwned>(&self, path: String) -> Result<Arc<T>, TenorError> {
        let response = self
            .client
            .get(format!("{TENOR_API_BASE_URL}{path}"))
            .send()
            .await
            .inspect_err(|e| {
                revolt_config::capture_error(e);
            })
            .map_err(|_| TenorError::HttpError)?;

        let text = response.text().await.map_err(|e| {
            println!("{e:?}");
            TenorError::HttpError
        })?;

        Ok(Arc::new(serde_json::from_str(&text).unwrap()))
    }

    pub async fn search(
        &self,
        query: &str,
        locale: &str,
        limit: u32,
        is_category: bool,
        position: &str,
    ) -> Result<Arc<types::PaginatedMediaResponse>, TenorError> {
        let unique_key = format!("s:{query}:{locale}:{is_category}:{position}");

        if self.cache.read().await.contains_key(&unique_key) {
            if let Some(response) = self.cache.write().await.get(&unique_key) {
                return Ok(response.clone());
            }
        }

        let res = self.coalescion.execute(unique_key.clone(), || {
            let mut path = format!(
                "/search?key={}&q={}&client_key=Gifbox&media_filter=webm,tinywebm&locale={}&contentfilter=high&limit={limit}",
                &self.key,
                url_encode(query),
                url_encode(locale),
            );

            if !position.is_empty() {
                path.push_str("&pos=");
                path.push_str(position);
            };

            if is_category {
                path.push_str("&component=categories");
            }

            self.request::<types::PaginatedMediaResponse>(path)
        })
        .await
        .unwrap();

        if let Ok(resp) = &*res {
            self.cache.write().await.insert(unique_key, resp.clone());
        }

        (*res).clone()
    }

    pub async fn categories(
        &self,
        locale: &str,
    ) -> Result<Arc<types::CategoriesResponse>, TenorError> {
        let unique_key = format!("c-{locale}");

        if self.categories.read().await.contains_key(&unique_key) {
            if let Some(response) = self.categories.write().await.get(&unique_key) {
                return Ok(response.clone());
            }
        }

        let res = self
            .coalescion
            .execute(unique_key.clone(), || {
                let path = format!(
                    "/categories?key={}&client_key=Gifbox&locale={}&contentfilter=high",
                    &self.key,
                    url_encode(locale),
                );

                self.request::<types::CategoriesResponse>(path)
            })
            .await
            .unwrap();

        if let Ok(resp) = &*res {
            self.categories
                .write()
                .await
                .insert(unique_key, resp.clone());
        }

        (*res).clone()
    }

    pub async fn featured(
        &self,
        locale: &str,
        limit: u32,
        position: &str,
    ) -> Result<Arc<types::PaginatedMediaResponse>, TenorError> {
        let unique_key = format!("f-{locale}-{limit}-{position}");

        if self.categories.read().await.contains_key(&unique_key) {
            if let Some(response) = self.featured.write().await.get(&unique_key) {
                return Ok(response.clone());
            }
        }

        let res = self.coalescion.execute(unique_key.clone(), || {
            let mut path = format!(
                "/featured?key={}&client_key=Gifbox&media_filter=webm,tinywebm&locale={}&contentfilter=high&limit={limit}",
                &self.key,
                url_encode(locale),
            );

            if !position.is_empty() {
                path.push_str("&pos=");
                path.push_str(position);
            };

            self.request::<types::PaginatedMediaResponse>(path)
        })
        .await
        .unwrap();

        if let Ok(resp) = &*res {
            self.featured.write().await.insert(unique_key, resp.clone());
        }

        (*res).clone()
    }
}
