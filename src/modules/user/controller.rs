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
    modules::{
        ApiModule,
        user::domain::{CreateUserRequest, UserResponse},
    },
};

pub struct UserModule;

impl ApiModule for UserModule {
    fn routes() -> Router<AppState> {
        Router::new()
            .route("/users", get(list_users).post(create_user))
            .route("/users/{id}", get(get_user))
    }
}

async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    req.validate().map_err(|err| {
        tracing::warn!(email = %req.email, error = %err, "invalid create user request");
        AppError::Validation(err.to_string())
    })?;

    let result = state.user_service.create(req).await?;

    Ok(Json(ApiResponse::new(result)))
}

async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<Option<UserResponse>>>> {
    let result = state.user_service.get_by_id(id).await?;

    Ok(Json(ApiResponse::new(result)))
}

async fn list_users(
    State(state): State<AppState>,
) -> AppResult<Json<ApiResponse<Vec<UserResponse>>>> {
    let result = state.user_service.get_list().await?;

    Ok(Json(ApiResponse::new(result)))
}
