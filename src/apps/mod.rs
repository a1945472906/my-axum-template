use axum::Router;

// pub mod system;
// pub mod user;
pub mod app;

trait View {
    fn as_route() -> Router;
}
