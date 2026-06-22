use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};

use crate::{
    core::{app_state::AppState, error::AppResult, response::ApiResponse},
    modules::user::domain::UserResponse,
};

pub struct UserModule;

impl UserModule {
    pub fn routes() -> Router<AppState> {
        Router::new().route("/users/{id}", get(get_user))
    }
}

async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<Option<UserResponse>>>> {
    let result = state.user_service.get_by_id(id).await?;

    Ok(Json(ApiResponse::new(result)))
}
