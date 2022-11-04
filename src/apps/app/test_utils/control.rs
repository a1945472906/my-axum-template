use super::model::{req::*, res::*}; 
use utils::libs::{lru_k::LRUKCache,response::{ErrCode, Meta},rc::Ptr};

pub async fn put_lru_2_cache(req: PutLru2CacheReq, mut cache: Ptr<LRUKCache<String, u8>>) -> Result<(), Meta> {
    cache.put(req.key, req.value);
    Ok(())
}

pub async fn get_lru2_cache(req: GetLru2CacheReq, mut cache: Ptr<LRUKCache<String, u8>>) -> Result<GetLru2CacheRes, Meta> {
    match cache.get(&req.key) {
        Some(value) => Ok(GetLru2CacheRes{
            key: req.key.clone(), 
            value: *value
        }),
        None => {
            Err(Meta::default())
        }
    }
}