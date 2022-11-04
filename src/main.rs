#![feature(drain_filter)]
#[allow(unused)]
mod apps;
use apps::app::{user::model::model::UserInfo, self};
use axum::{extract::Extension, http::StatusCode, middleware, routing::get_service, Router};
use std::{collections::HashMap, net::SocketAddr, ptr::NonNull};
use futures;
use tokio::time::Duration;
use utils::libs::{
    extension::Cache,
    extract,
    global::{self, Observable},
    jwt::{Token},
    rc::CancerCell,
    email::{
        mail::EmailServer, 
        model::email_mark::*,
    },
    db::{database::DB,model::db_mark::*}, 
    lru_k::LRUKCache
};

pub type RefreshTokenCache = Cache<String, Token<UserInfo>>;
fn init() {
    let env = global::get_global_env();
    let content_limit = extract::get().get_mut();
    env.watch(|new_value| {
        *content_limit = match new_value.get("CONTENT_LENGTH_LIMIT").unwrap().as_str() {
            "large" => 2048 * 1024 * 1024,  //  2g
            "medium" => 1024 * 1024 * 1024, //  1g
            "small" => 512 * 1024 * 1024,   //  512m
            _ => 0,
        };
        println!("content_length_limit change: {}", *content_limit);
    })
}

const CLEAN_TASK_TICK: u64 = 3600 * 24;

#[tokio::main]
async fn main() {
    init();
    let db = DB::<User>::new(
        global::get_global_env().get("DATABASE_URL").unwrap(),
    )
    .await;
    let c = CancerCell::new(HashMap::new());
    let cache: RefreshTokenCache = Cache(c.get_mut_raw());
    let lru_cache:CancerCell<LRUKCache<String,u8>> = CancerCell::new(LRUKCache::new(2, 2048, 2048));
    let mut cache_clone = cache.clone();
    tokio::spawn(async move {
        let interval = tokio::time::interval(Duration::from_secs(CLEAN_TASK_TICK));
        cache_clone.clean_task(interval).await;
    });
    let app = Router::new()
        .merge(apps::app::as_route())
        .layer(Extension(db))
        .layer(Extension(cache))
        .layer(Extension(lru_cache.get_ptr()));

        // .layer(Extension(email_server_sender));
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let serve = axum::Server::bind(&addr).serve(app.into_make_service());
    let _ = futures::join!(serve);
}
