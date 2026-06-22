use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, header, request::Parts},
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
        let token = extract_bearer_token(&parts.headers)?;
        let claims = state.auth_service.verify_access_token(token).await?;

        Ok(CurrentUser {
            user_id: claims.sub,
            role: claims.role,
        })
    }
}

pub fn extract_bearer_token(headers: &HeaderMap) -> AppResult<String> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?;

    if token.trim().is_empty() {
        return Err(AppError::Unauthorized);
    }

    Ok(token.to_string())
}
