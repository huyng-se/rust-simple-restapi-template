use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use validator::Validate;

use crate::{
    core::{
        app_state::AppState,
        error::{AppError, AppResult},
        response::ApiResponse,
    },
    infra::middleware::extractor::CredentialContext,
    modules::user::user_domain::{UpdateUserRequest, UserResponse},
};

pub struct UserModule;

impl UserModule {
    pub fn routes() -> Router<AppState> {
        Router::new().route("/users/{id}", get(get_user).patch(update_user))
    }

    pub fn admin_routes() -> Router<AppState> {
        Router::new().route("/admin/users", get(list_users))
    }
}

async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<Option<UserResponse>>>> {
    let result = state.user_service.get_by_id(id).await?;

    Ok(Json(ApiResponse::new(result)))
}

async fn update_user(
    State(state): State<AppState>,
    cred: CredentialContext,
    Path(id): Path<i64>,
    Json(req): Json<UpdateUserRequest>,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    let user_id = cred
        .user_id
        .parse::<i64>()
        .map_err(|_| AppError::Internal)?;
    if user_id != id {
        return Err(AppError::Forbidden);
    }

    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    let user = state.user_service.update(id, req).await?;
    Ok(Json(ApiResponse::new(user)))
}

async fn list_users(
    State(state): State<AppState>,
) -> AppResult<Json<ApiResponse<Vec<UserResponse>>>> {
    let result = state.user_service.get_list().await?;

    Ok(Json(ApiResponse::new(result)))
}
