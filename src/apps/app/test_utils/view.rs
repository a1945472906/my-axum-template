use axum::{
    extract::{Path, Query},
    response::IntoResponse,
    routing::{get, post, Router},
    Extension, Json,
};
use utils::libs::{
    lru_k::LRUKCache,
    mongo::{mongo_mark, MongoDB},
    rc::Ptr,
    redis::{
        redis_mark::{Master, Slave},
        redis_mode::{R, W},
        RedisPool,
    },
    response::{Meta, Response},
};

use super::control;
// use super::model::{model::UserInfo, req::*};
use super::model::{req::*, res::*};
use crate::apps::View;
use crate::RefreshTokenCache;
// pub struct UserView;
pub struct TestUtilView;

impl TestUtilView {
    async fn put_lru_2_cache(
        Json(req): Json<PutLru2CacheReq>,
        Extension(mut cache): Extension<Ptr<LRUKCache<String, u8>>>,
    ) -> impl IntoResponse {
        Response::from(control::put_lru_2_cache(req, cache).await)
    }
    async fn get_lru2_cache(
        Query(req): Query<GetLru2CacheReq>,
        Extension(mut cache): Extension<Ptr<LRUKCache<String, u8>>>,
    ) -> impl IntoResponse {
        Response::from(control::get_lru2_cache(req, cache).await)
    }
    async fn get_redis(
        Query(req): Query<GetRedisReq>,
        Extension(mut redis_conn): Extension<Ptr<RedisPool<Slave, R>>>,
    ) -> impl IntoResponse {
        Response::from(control::redis_get(req, redis_conn).await)
    }
    async fn put_redis(
        Json(req): Json<PutRedisReq>,
        Extension(mut redis_conn): Extension<Ptr<RedisPool<Master, W>>>,
    ) -> impl IntoResponse {
        Response::from(control::redis_put(req, redis_conn).await)
    }
    async fn put_mongodb(
        Json(req): Json<PutMongoDBReq>,
        Extension(mongodb_conn): Extension<Ptr<MongoDB<mongo_mark::Master>>>,
    ) -> impl IntoResponse {
        Response::from(control::test_mongodb_insert(req, mongodb_conn).await)
    }
    async fn find_mongodb(
        Query(req): Query<FindMongoDBReq>,
        Extension(mongodb_conn): Extension<Ptr<MongoDB<mongo_mark::Master>>>,
    ) -> impl IntoResponse {
        Response::from(control::test_mongodb_find(req, mongodb_conn).await)
    }
}
impl View for TestUtilView {
    fn as_route() -> Router {
        let app = Router::new()
            .route(
                "/lru_2_cache",
                post(Self::put_lru_2_cache).get(Self::get_lru2_cache),
            )
            .route("/redis", post(Self::put_redis).get(Self::get_redis))
            .route("/mongodb", post(Self::put_mongodb).get(Self::find_mongodb));
        Router::new().nest("/test_utils", app)
    }
}
