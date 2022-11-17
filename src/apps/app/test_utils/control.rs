use super::model::{req::*, res::*};
use utils::libs::{
    lru_k::LRUKCache,
    rc::Ptr,
    redis::{
        redis_mark::{Master, Slave},
        redis_mode::{R, W},
        Commands, RedisPool,
    },
    response::{ErrCode, Meta},
};

pub async fn put_lru_2_cache(
    req: PutLru2CacheReq,
    mut cache: Ptr<LRUKCache<String, u8>>,
) -> Result<(), Meta> {
    cache.put(req.key, req.value);
    Ok(())
}

pub async fn get_lru2_cache(
    req: GetLru2CacheReq,
    mut cache: Ptr<LRUKCache<String, u8>>,
) -> Result<GetLru2CacheRes, Meta> {
    match cache.get(&req.key) {
        Some(value) => Ok(GetLru2CacheRes {
            key: req.key.clone(),
            value: *value,
        }),
        None => Err(Meta::default()),
    }
}

pub async fn redis_get(
    req: GetRedisReq,
    redis_conn: Ptr<RedisPool<Slave, R>>,
) -> Result<GetRedisRes, Meta> {
    let mut conn = redis_conn.conn();
    // let value: String = conn.get(req.key).unwrap();
    match conn.get(&req.key) {
        Ok(value) => Ok(GetRedisRes {
            key: req.key,
            value,
        }),
        Err(e) => Err(Meta::from(400, &e.to_string())),
    }
    // Ok(GetRedisRes {})
}

pub async fn redis_put(
    req: PutRedisReq,
    redis_conn: Ptr<RedisPool<Master, W>>,
) -> Result<(), Meta> {
    let mut conn = redis_conn.conn();
    match conn.set(&req.key, &req.value) {
        Ok(()) => Ok(()),
        Err(e) => Err(Meta::from(400, &e.to_string())),
    }
}
