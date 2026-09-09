pub mod api;
pub mod telegram;

use axum::Router;

pub fn create_server(state: api::AppState) -> Router {
    api::create_router(state)
}
