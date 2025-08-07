use std::{collections::{HashMap}, fmt::Debug, hash::Hash, sync::Arc, future::Future};

use tokio::{sync::{watch::{channel as watch_channel, Receiver}, RwLock, Mutex}};

#[cfg(feature = "cache")]
use lru::LruCache;

#[cfg(feature = "queue")]
use indexmap::IndexMap;

use crate::{CoalescionServiceConfig, Error};

#[derive(Clone, Debug)]
#[allow(clippy::type_complexity)]
pub struct CoalescionService<Id: Hash + Eq, Value> {
    config: Arc<CoalescionServiceConfig>,
    watchers: Arc<RwLock<HashMap<Id, Receiver<Option<Result<Arc<Value>, Error>>>>>>,
    #[cfg(feature = "queue")]
    queue: Arc<RwLock<IndexMap<Id, Receiver<Option<Result<Arc<Value>, Error>>>>>>,
    #[cfg(feature = "cache")]
    cache: Option<Arc<Mutex<LruCache<Id, Arc<Value>>>>>,
}

impl<Id: Hash + PartialEq + Eq + Clone + Ord, Value> CoalescionService<Id, Value> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_config(config: CoalescionServiceConfig) -> Self {
        Self {
            config: Arc::new(config),
            watchers: Arc::new(RwLock::new(HashMap::new())),
            #[cfg(feature = "queue")]
            queue: Arc::new(RwLock::new(IndexMap::new())),
            #[cfg(feature = "cache")]
            cache: None,
        }
    }

    #[cfg(feature = "cache")]
    pub fn from_cache(config: CoalescionServiceConfig, cache: LruCache<Id, Arc<Value>>) -> Self {
        Self {
            cache: Some(Arc::new(Mutex::new(cache))),
            ..Self::from_config(config)
        }
    }

    async fn wait_for(&self, mut receiver: Receiver<Option<Result<Arc<Value>, Error>>>) -> Result<Arc<Value>, Error> {
        receiver
            .wait_for(|v| v.is_some())
            .await
            .map_err(|_| Error::RecvError)
            .and_then(|r| r.clone().unwrap())
    }

    async fn insert_and_execute<F: FnOnce() -> Fut, Fut: Future<Output = Value>>(&self, id: Id, func: F) -> Result<Arc<Value>, Error> {
        let (send, recv) = watch_channel(None);

        self.watchers.write().await.insert(id.clone(), recv);

        let value = Ok(Arc::new(func().await));

        send.send_modify(|opt| {
            opt.replace(value.clone());
        });

        #[cfg(feature = "cache")]
        if let Some(cache) = self.cache.as_ref() {
            if let Ok(value) = &value {
                cache.lock().await.push(id.clone(), value.clone());
            }
        };

        self.watchers.write().await.remove(&id);

        value
    }

    pub async fn execute<F: FnOnce() -> Fut, Fut: Future<Output = Value>>(&self, id: Id, func: F) -> Result<Arc<Value>, Error> {
        #[cfg(feature = "cache")]
        if let Some(cache) = self.cache.as_ref() {
            if let Some(value) = cache.lock().await.get(&id) {
                return Ok(value.clone())
            }
        };

        let (receiver, length) = {
            let watchers = self.watchers.read().await;
            let length = watchers.len();

            (watchers.get(&id).cloned(), length)
        };

        if let Some(receiver) = receiver {
            self.wait_for(receiver).await
        } else {
            match self.config.max_concurrent {
                Some(max_concurrent) if length >= max_concurrent => {
                    #[cfg(feature = "queue")]
                    if self.config.queue_requests {
                        let (receiver, length) = {
                            let queue = self.queue.read().await;

                            (queue.get(&id).cloned(), queue.len())
                        };

                        if let Some(receiver) = receiver {
                            return self.wait_for(receiver).await
                        } else {
                            if self.config.max_queue.is_some_and(|max_queue| max_queue >= length) {
                                return Err(Error::MaxQueue)
                            };

                            let (send, recv) = watch_channel(None);

                            self.queue.write().await.insert(id.clone(), recv);

                            loop {
                                let length = self.watchers.read().await.len();

                                if length < max_concurrent {
                                    let first_key = {
                                        let queue = self.queue.read().await;
                                        queue.first().map(|v| v.0).cloned()
                                    };

                                    if first_key == Some(id.clone()) {
                                        self.queue.write().await.shift_remove(&id);

                                        let response = self.insert_and_execute(id, func).await;

                                        send.send_modify(|opt| {
                                            opt.replace(response.clone());
                                        });

                                        return response
                                    }
                                }
                            }
                        }
                    } else {
                        Err(Error::MaxConcurrent)
                    }

                    #[cfg(not(feature = "queue"))]
                    Err(Error::MaxConcurrent)
                }
                _ => {
                    self.insert_and_execute(id, func).await
                }
            }
        }
    }

    pub async fn current_task_count(&self) -> usize {
        self.watchers.read().await.len()
    }

    pub async fn current_queue_len(&self) -> usize {
        self.queue.read().await.len()
    }
}

impl<Id: Hash + PartialEq + Eq + Clone + Ord, Value> Default for CoalescionService<Id, Value> {
    fn default() -> Self {
        Self::from_config(CoalescionServiceConfig::default())
    }
}
