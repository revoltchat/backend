use crate::util::variables::REDIS_URI;

use mobc::Pool;
use mobc_redis::RedisConnectionManager;
use once_cell::sync::OnceCell;

static REDISPOOL: OnceCell<Pool<RedisConnectionManager>> = OnceCell::new();

pub async fn connect() {
    let client = mobc_redis::redis::Client::open(REDIS_URI.to_string()).unwrap();
    let manager = mobc_redis::RedisConnectionManager::new(client);
    let pool = mobc::Pool::builder().max_open(100).build(manager);
    REDISPOOL.set(pool).ok().unwrap();
}

pub fn get_pool() -> &'static Pool<RedisConnectionManager> {
    REDISPOOL.get().unwrap()
}
