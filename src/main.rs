#![feature(drain_filter)]
#[allow(unused)]
mod apps;
use apps::app::user::model::model::UserInfo;
use axum::{
    // extract::{Extension, FromRef, State},
    extract::FromRef,
    Router,
};
use futures;
use std::{collections::HashMap, net::SocketAddr};
use tokio::time::Duration;
use utils::libs::{
    db::{database::DB, model::db_mark::*},
    // email::{mail::EmailServer, model::email_mark::*},
    extension::Cache,
    extract,
    global::{self, Observable},
    jwt::Token,
    lru_k::LRUKCache,
    mongo::{mongo_mark, MongoDB},
    rc::{CancerCell, Ptr},
    redis::{
        redis_mark::{Master, Slave},
        redis_mode::{R, W},
        RedisPool,
    },
    tracing::{default_shutdown_signal, TraceInit},
};
// use std::collections::HashMap;
use tower_http::trace::TraceLayer;

pub type RefreshTokenCache = Cache<String, Token<UserInfo>>;

// #[derive(Clone)]
pub struct AppState {
    user_db: Ptr<DB<User>>,
    cache: RefreshTokenCache,
    lru_cache: Ptr<LRUKCache<String, u8>>,
    redis_master: Ptr<RedisPool<Master, W>>,
    redis_slave: Ptr<RedisPool<Slave, R>>,
    mongo_db: Ptr<MongoDB<mongo_mark::Master>>,
}
impl FromRef<AppState> for Ptr<DB<User>> {
    fn from_ref(state: &AppState) -> Self {
        state.user_db.clone()
    }
}
impl FromRef<AppState> for RefreshTokenCache {
    fn from_ref(input: &AppState) -> Self {
        input.cache.clone()
    }
}
impl FromRef<AppState> for Ptr<LRUKCache<String, u8>> {
    fn from_ref(input: &AppState) -> Self {
        input.lru_cache.clone()
    }
}

impl FromRef<AppState> for Ptr<RedisPool<Master, W>> {
    fn from_ref(input: &AppState) -> Self {
        input.redis_master.clone()
    }
}

impl FromRef<AppState> for Ptr<RedisPool<Slave, R>> {
    fn from_ref(input: &AppState) -> Self {
        input.redis_slave.clone()
    }
}
impl FromRef<AppState> for Ptr<MongoDB<mongo_mark::Master>> {
    fn from_ref(input: &AppState) -> Self {
        input.mongo_db.clone()
    }
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for AppState {}

fn init() {
    let env = global::get_global_env();
    let content_limit = extract::get().get_mut();
    env.watch(|_, new_value| {
        *content_limit = match new_value.get("CONTENT_LENGTH_LIMIT").unwrap().as_str() {
            "large" => 2048 * 1024 * 1024,  //  2g
            "medium" => 1024 * 1024 * 1024, //  1g
            "small" => 512 * 1024 * 1024,   //  512m
            _ => 0,
        };
        tracing::info!("content_length_limit change: {}", *content_limit);
    });
    TraceInit::default().init();
}

const CLEAN_TASK_TICK: u64 = 3600 * 24;

#[tokio::main]
async fn main() {
    init();
    let db = CancerCell::new(
        DB::<User>::new(global::get_global_env().get("DATABASE_URL").unwrap()).await,
    );
    let c = CancerCell::new(HashMap::new());
    let cache: RefreshTokenCache = Cache(c.get_mut_raw());
    let lru_cache: CancerCell<LRUKCache<String, u8>> =
        CancerCell::new(LRUKCache::new(2, 2048, 2048));
    let mut cache_clone = cache.clone();
    let redis_master = CancerCell::new(RedisPool::<Master, W>::new(
        global::get_global_env().get("REDIS_MASTER").unwrap(),
    ));
    let redis_slave = CancerCell::new(RedisPool::<Slave, R>::new(
        global::get_global_env().get("REDIS_SLAVE").unwrap(),
    ));
    let mongo_client = CancerCell::new(
        MongoDB::<mongo_mark::Master>::new(
            global::get_global_env().get("MONGODB_EXAMPLE").unwrap(),
            "myapp",
        )
        .await,
    );

    // let mongodb = CancerCell::new(mongo_client);
    tokio::spawn(async move {
        let interval = tokio::time::interval(Duration::from_secs(CLEAN_TASK_TICK));
        cache_clone.clean_task(interval).await;
    });
    let appstate = AppState {
        user_db: db.get_ptr(),
        cache,
        lru_cache: lru_cache.get_ptr(),
        redis_master: redis_master.get_ptr(),
        redis_slave: redis_slave.get_ptr(),
        mongo_db: mongo_client.get_ptr(),
    };
    let app = Router::new()
        .merge(apps::app::as_route())
        .with_state(appstate)
        .layer(TraceLayer::new_for_http());

    // .layer(Extension(db))
    // .layer(Extension(cache))
    // .layer(Extension(lru_cache.get_ptr()))
    // .layer(Extension(redis_master.get_ptr()))
    // .layer(Extension(redis_slave.get_ptr()))
    // .layer(Extension(mongodb.get_ptr()));
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let serve = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(default_shutdown_signal());
    let _ = futures::join!(serve);
}
