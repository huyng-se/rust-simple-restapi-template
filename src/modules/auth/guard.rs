use crate::{
    core::{
        app_state::AppState,
        error::{AppError, AppResult},
    },
    modules::auth::extractor::AuthContext,
};
use axum::{
    extract::{Request, State},
    http::{
        HeaderMap,
        header::{self},
    },
    middleware::Next,
    response::Response,
};

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> AppResult<Response> {
    let token = extract_bearer_token(&req.headers())?;
    let claims = state
        .auth_service
        .verify_access_token(token.clone()) // TODO: fix clone later
        .await?;

    let user_id = claims
        .sub
        .parse::<i64>()
        .map_err(|_| AppError::Unauthorized)?;

    req.extensions_mut().insert(AuthContext {
        user_id: user_id.to_string(),
        role: claims.role,
        access_token: token,
        access_jti: claims.jti,
    });

    Ok(next.run(req).await)
}

pub async fn require_admin(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> AppResult<Response> {
    let token = extract_bearer_token(&req.headers())?;
    let claims = state
        .auth_service
        .verify_access_token(token.clone()) // TODO: fix clone later
        .await?;

    if claims.role != "ADMIN" {
        return Err(AppError::Forbidden);
    }

    let user_id = claims
        .sub
        .parse::<i64>()
        .map_err(|_| AppError::Unauthorized)?;

    req.extensions_mut().insert(AuthContext {
        user_id: user_id.to_string(),
        role: claims.role,
        access_token: token,
        access_jti: claims.jti,
    });

    Ok(next.run(req).await)
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
