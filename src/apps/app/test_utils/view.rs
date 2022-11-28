use super::control;
use axum::{
    debug_handler,
    extract::{Json, Path, Query, State, TypedHeader, WebSocketUpgrade},
    response::IntoResponse,
    routing::{get, post, Router},
    Extension,
};
use http::StatusCode;
use serde::{Deserialize, Serialize};
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
// use super::model::{model::UserInfo, req::*};
use super::model::{req::*, res::*};
use crate::apps::View;
use crate::AppState;
use crate::RefreshTokenCache;
use std::marker::PhantomData;
use std::sync::Arc;
// pub struct UserView;
// use axum::extract::State;

pub struct TestUtilView;

impl TestUtilView {
    async fn put_lru_2_cache(
        State(_): State<AppState>,
        State(mut cache): State<Ptr<LRUKCache<String, u8>>>,
        Json(req): Json<PutLru2CacheReq>,
    ) -> impl IntoResponse {
        Response::from(control::put_lru_2_cache(req, cache).await)
        // (StatusCode::ACCEPTED, "aaa")
    }
    async fn get_lru2_cache(
        Query(req): Query<GetLru2CacheReq>,
        State(mut cache): State<Ptr<LRUKCache<String, u8>>>,
    ) -> impl IntoResponse {
        Response::from(control::get_lru2_cache(req, cache).await)
    }
    async fn get_redis(
        Query(req): Query<GetRedisReq>,
        State(mut redis_conn): State<Ptr<RedisPool<Slave, R>>>,
    ) -> impl IntoResponse {
        Response::from(control::redis_get(req, redis_conn).await)
    }
    async fn put_redis(
        State(mut redis_conn): State<Ptr<RedisPool<Master, W>>>,
        Json(req): Json<PutRedisReq>,
    ) -> impl IntoResponse {
        Response::from(control::redis_put(req, redis_conn).await)
    }
    // #[debug_handler]
    async fn put_mongodb(
        // State(mongodb_conn): State<Ptr<MongoDB<mongo_mark::Master>>>,
        State(mongodb_conn): State<Ptr<MongoDB<mongo_mark::Master>>>,
        Json(req): Json<PutMongoDBReq>,
    ) -> impl IntoResponse {
        // Json(())
        Response::from(control::test_mongodb_insert(req, mongodb_conn).await)
    }
    async fn find_mongodb(
        Query(req): Query<FindMongoDBReq>,
        State(mongodb_conn): State<Ptr<MongoDB<mongo_mark::Master>>>,
    ) -> impl IntoResponse {
        Response::from(control::test_mongodb_find(req, mongodb_conn).await)
    }
    async fn test_websocket(
        ws: WebSocketUpgrade,
        user_agent: Option<TypedHeader<headers::UserAgent>>,
    ) -> impl IntoResponse {
        if let Some(TypedHeader(user_agent)) = user_agent {
            println!("`{}` connected", user_agent.as_str());
        }

        ws.on_upgrade(control::ws_handle)
    }
}
impl View for TestUtilView {
    fn as_route() -> Router<AppState> {
        let app = Router::new()
            // .with_state(PhantomData::<AppState>)
            .route(
                "/lru_2_cache",
                post(Self::put_lru_2_cache).get(Self::get_lru2_cache),
            )
            .route("/redis", post(Self::put_redis).get(Self::get_redis))
            .route("/mongodb", get(Self::find_mongodb).post(Self::put_mongodb))
            .route("/ws", get(Self::test_websocket));
        Router::new().nest("/test_utils", app)
    }
}
