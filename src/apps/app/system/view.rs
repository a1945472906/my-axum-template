use crate::apps::View;
use axum::{response::IntoResponse, routing::*, Json, Router};
use std::collections::HashMap;
use utils::libs::{
    global,
    response::{Meta, Response},
};
pub struct SystemView;
impl SystemView {
    async fn set_env(Json(req): Json<HashMap<String, String>>) -> impl IntoResponse {
        let env_map = global::get_global_env();
        env_map.extend(req);
        unsafe {
            env_map.notify();
        };
        let res = Response {
            meta: Meta::default(),
            body: Some(()),
        };
        res
    }
    async fn get_env() -> impl IntoResponse {
        let env_map = global::get_global_env();
        let res = Response {
            meta: Meta::default(),
            body: Some(env_map.get_value()),
        };
        res
    }
}

impl View for SystemView {
    fn as_route() -> Router {
        let app = Router::new().route("/env", get(Self::get_env).put(Self::set_env));
        Router::new().nest("/sys", app)
    }
}
