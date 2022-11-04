use axum::{response::IntoResponse, routing::{post,get, Router}, Json, Extension,extract::{
    Query,
    Path
} };
use utils::libs::{
    lru_k::LRUKCache,
    response::{Meta, Response},
    rc::Ptr
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
        Extension(mut cache): Extension<Ptr<LRUKCache<String, u8>>>
    ) -> impl IntoResponse {
        Response::from(control::put_lru_2_cache(req, cache).await)
    }
    async fn get_lru2_cache(
        Query(req): Query<GetLru2CacheReq>, 
        Extension(mut cache): Extension<Ptr<LRUKCache<String, u8>>>
    ) -> impl IntoResponse {
        Response::from(control::get_lru2_cache(req, cache).await)
    }

}
impl View for TestUtilView {
    fn as_route() -> Router {
        let app = Router::new()
            .route("/lru_2_cache", post(Self::put_lru_2_cache).get(Self::get_lru2_cache)
        );
        Router::new().nest("/test_utils", app)
    }
}