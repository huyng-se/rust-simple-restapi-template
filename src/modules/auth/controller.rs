use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use validator::Validate;

use crate::{
    core::{
        app_state::AppState,
        error::{AppError, AppResult},
        response::ApiResponse,
    },
    modules::{
        auth::{
            domain::{
                AuthResponse, LoginRequest, LogoutRequest, MessageResponse, RefreshTokenRequest,
                RegisterRequest,
            },
            extractor::AuthContext,
        },
        user::domain::UserResponse,
    },
};

pub struct AuthModule;

impl AuthModule {
    pub fn public_routes() -> Router<AppState> {
        Router::new()
            .route("/register", post(register))
            .route("/login", post(login))
            .route("/refresh", post(refresh))
    }

    pub fn protected_routes() -> Router<AppState> {
        Router::new()
            .route("/logout", post(logout))
            .route("/me", get(me))
    }
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<ApiResponse<AuthResponse>>> {
    req.validate().map_err(|err| {
        tracing::warn!(email = %req.email, error = %err, "invalid register request");
        AppError::Validation(err.to_string())
    })?;

    let token = state.auth_service.register(req).await?;

    Ok(Json(ApiResponse::new(token)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<ApiResponse<AuthResponse>>> {
    req.validate().map_err(|err| {
        tracing::warn!(email = %req.email, error = %err, "invalid login request");
        AppError::Validation(err.to_string())
    })?;

    let token = state.auth_service.login(req).await?;

    Ok(Json(ApiResponse::new(token)))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshTokenRequest>,
) -> AppResult<Json<ApiResponse<AuthResponse>>> {
    if req.refresh_token.trim().is_empty() {
        return Err(AppError::Validation(
            "refresh_token is required".to_string(),
        ));
    }

    let token = state.auth_service.refresh(req.refresh_token).await?;

    Ok(Json(ApiResponse::new(token)))
}

pub async fn logout(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(req): Json<LogoutRequest>,
) -> AppResult<Json<ApiResponse<MessageResponse>>> {
    if req.refresh_token.trim().is_empty() {
        return Err(AppError::Validation(
            "refresh_token is required".to_string(),
        ));
    }

    state
        .auth_service
        .logout(auth.access_token, req.refresh_token)
        .await?;

    Ok(Json(ApiResponse::new(MessageResponse {
        message: "Logged out successfully.".to_string(),
    })))
}

pub async fn me(
    State(state): State<AppState>,
    current_user: AuthContext,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    let user_id = current_user
        .user_id
        .parse::<i64>()
        .map_err(|_| AppError::Internal)?;

    let user = state.auth_service.me(user_id).await?;

    Ok(Json(ApiResponse::new(user)))
}
