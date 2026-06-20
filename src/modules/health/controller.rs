use axum::{Json, Router, extract::State, routing::get};
use diesel::select;
use diesel_async::RunQueryDsl;
use serde_json::json;

use crate::{
    core::{app_state::AppState, error::AppResult},
    modules::ApiModule,
};

pub struct HealthModule;

impl ApiModule for HealthModule {
    fn routes() -> Router<AppState> {
        Router::new()
            .route("/health/live", get(live))
            .route("/health/ready", get(ready))
    }
}

async fn live() -> AppResult<Json<serde_json::Value>> {
    Ok(Json(json!({
        "status": "ok"
    })))
}

async fn ready(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    let mut conn = state.db_pool.get().await?;

    select(diesel::dsl::sql::<diesel::sql_types::Integer>("1"))
        .execute(&mut conn)
        .await?;

    Ok(Json(json!({
        "status": "ready"
    })))
}
