use crate::core::{
    app_state::AppState,
    error::{AppError, AppResult},
};
use axum::{extract::FromRequestParts, http::request::Parts};

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub role: String,
    pub access_token: String,
    pub access_jti: String,
}

impl FromRequestParts<AppState> for AuthContext {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &AppState) -> AppResult<Self> {
        parts
            .extensions
            .get::<AuthContext>()
            .cloned()
            .ok_or(AppError::Unauthorized)
    }
}
