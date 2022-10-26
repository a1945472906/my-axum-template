use deadpool_redis::{
    redis::{cmd, FromRedisValue, RedisError},
    Config, Connection, Manager, Pool, PoolError, Runtime,
};
use std::marker::PhantomData;
// use crate::utils::global;

pub struct RedisPool<T> {
    pub pool: Pool,
    _marker: PhantomData<T>,
}
pub struct Conn(Connection);
impl Conn {
    pub async fn set(&mut self, key: &str, value: &str) -> Result<(), RedisError> {
        cmd("SET")
            .arg(&[key, value])
            .query_async::<_, ()>(&mut self.0)
            .await
    }
    pub async fn get(&mut self, key: &str) -> Result<String, RedisError> {
        cmd("GET").arg(&[key]).query_async(&mut self.0).await
    }
}

impl<T> RedisPool<T> {
    pub async fn new(redis_config: &'static str) -> Self {
        let cfg = Config::from_url(redis_config);
        let pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();
        Self {
            pool,
            _marker: PhantomData,
        }
    }
    pub async fn get_conn(&self) -> Result<Connection, PoolError> {
        self.pool.get().await
    }
}
