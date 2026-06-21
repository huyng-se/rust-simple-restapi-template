use crate::core::app_state::AppState;
use axum::Router;

pub mod auth;
pub mod health;
pub mod user;

pub trait ApiModule {
    fn routes() -> Router<AppState>;
}
