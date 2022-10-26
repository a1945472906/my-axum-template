pub mod system;
pub mod user;
use axum::{routing::Route, Router};
use system::view::SystemView;
use user::view::UserView;

use super::View;

// pub struct DefaultApp;
// impl App for DefaultApp {
//     fn as_route() -> Route {
//         Router::new().route(SystemView::)
//     }
// }
// pub struct DefaultApp; 

// impl App for DefaultApp {
pub fn as_route() -> Router {
    Router::new().merge(SystemView::as_route()).merge(UserView::as_route())
}
// }/