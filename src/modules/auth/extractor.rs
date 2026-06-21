use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};

use crate::core::{
    app_state::AppState,
    error::{AppError, AppResult},
};

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub user_id: String,
    pub role: String,
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> AppResult<Self> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .ok_or(AppError::Unauthorized)?;

        let auth_str = auth_header.to_str().map_err(|_| AppError::Unauthorized)?;
        let token = auth_str
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        let claims = state
            .auth_service
            .verify_access_token(token.to_owned())
            .await?;

        Ok(CurrentUser {
            user_id: claims.sub,
            role: claims.role,
        })
    }
}
