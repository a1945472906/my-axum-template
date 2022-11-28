// use axum::Extension;
use axum::{
    debug_handler,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::*,
    Extension, Json,
};
use utils::libs::{
    db::{database::DB, model::db_mark::User},
    extension::Cache,
    jwt::Token,
    rc::Ptr,
    response::{Meta, Response},
};

use super::control::*;
use super::model::{model::UserInfo, req::*};
use crate::apps::View;
use crate::AppState;
use crate::RefreshTokenCache;
pub struct UserView;

impl UserView {
    async fn login(
        State(_): State<AppState>,
        State(db): State<Ptr<DB<User>>>,
        State(cache): State<RefreshTokenCache>,
        Json(req): Json<Login>,
    ) -> impl IntoResponse {
        // let db = &*db;
        Response::from(login(req, &db, cache).await)
    }

    async fn key_login(
        State(db): State<Ptr<DB<User>>>,
        State(cache): State<RefreshTokenCache>,
        Query(req): Query<KeyLogin>,
    ) -> impl IntoResponse {
        Response::from(key_login(req, &db, cache).await)
    }

    async fn get_user_info(user_info: Token<UserInfo>) -> impl IntoResponse {
        Response {
            meta: Meta::default(),
            body: Some(user_info),
        }
    }

    async fn refresh_token(
        State(cache): State<RefreshTokenCache>,
        Json(req): Json<RefreshToken>,
    ) -> impl IntoResponse {
        Response::from(refresh_token(req, cache).await)
    }

    async fn add_role(
        user_info: Token<UserInfo>,
        State(db): State<Ptr<DB<User>>>,
        Json(req): Json<AddRole>,
    ) -> impl IntoResponse {
        match add_role(user_info, req, &db).await {
            Ok(_) => Response {
                meta: Meta::default(),
                body: Some(()),
            },
            Err(meta) => Response { meta, body: None },
        }
    }

    async fn update_user_info(
        user_info: Token<UserInfo>,
        State(db): State<Ptr<DB<User>>>,
        State(cache): State<RefreshTokenCache>,
        Json(req): Json<UpdateUserInfo>,
    ) -> impl IntoResponse {
        match update_user_info(user_info, req, &db, cache).await {
            Ok(resp) => Response {
                meta: Meta::default(),
                body: Some(resp),
            },
            Err(meta) => Response { meta, body: None },
        }
    }

    async fn get_user(
        user_info: Token<UserInfo>,
        State(db): State<Ptr<DB<User>>>,
    ) -> impl IntoResponse {
        Response::from(get_user(user_info, &db).await)
    }
}

impl View for UserView {
    fn as_route() -> Router<AppState> {
        let app = Router::new()
            .route("/", get(Self::get_user))
            .route("/login", post(Self::login).get(Self::key_login))
            .route("/refresh_token", post(Self::refresh_token))
            .route("/user_info", get(Self::get_user_info))
            .route("/role", put(Self::add_role))
            .route("/update_user_info", put(Self::update_user_info));
        // .with_state(state);
        Router::new().nest("/user", app)
    }
}
