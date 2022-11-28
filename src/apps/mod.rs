use crate::AppState;
use axum::Router;
// pub mod system;
// pub mod user;
pub mod app;

trait View {
    // type State;
    fn as_route() -> Router<AppState>;
}
