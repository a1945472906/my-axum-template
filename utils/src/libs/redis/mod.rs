use r2d2::{Pool, PooledConnection};
use redis::Client;
pub use redis::Commands;
use std::marker::PhantomData;
pub struct RedisPool<T, S> {
    pub pool: Pool<Client>,
    _marker: PhantomData<(T, S)>,
}
impl<T, S> RedisPool<T, S> {
    pub fn new(redis_url: &'static str) -> Self {
        let client = Client::open(redis_url).expect("can not connect to redis");
        let pool = Pool::builder()
            .max_size(32)
            .build(client)
            .unwrap_or_else(|e| panic!("Error building redis pool:{}", e));
        Self {
            pool,
            _marker: PhantomData,
        }
    }
    pub fn conn(&self) -> PooledConnection<Client> {
        self.pool.get().unwrap()
    }
}

pub mod redis_mode {
    pub struct R;
    pub struct W;
}

pub mod redis_mark {
    pub struct Master;
    pub struct Slave;
}
