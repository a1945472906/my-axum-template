pub mod system;
pub mod test_utils;
pub mod user;
use super::View;
use crate::AppState;
use axum::{routing::Route, Router};
use system::view::SystemView;
use test_utils::view::TestUtilView;
use user::view::UserView;

// pub struct DefaultApp;
// impl App for DefaultApp {
//     fn as_route() -> Route {
//         Router::new().route(SystemView::)
//     }
// }
// pub struct DefaultApp;

// impl App for DefaultApp {
pub fn as_route() -> Router<AppState> {
    Router::new()
        .merge(SystemView::as_route())
        .merge(UserView::as_route())
        .merge(TestUtilView::as_route())
}
// }/
