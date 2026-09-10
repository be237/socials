pub mod api;
pub mod auth;
pub mod telegram;
pub mod whatsapp;

use axum::Router;

pub fn create_server(state: api::AppState) -> Router {
    api::create_router(state)
}
