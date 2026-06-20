use axum::{
    Router,
    http::{HeaderValue, StatusCode},
};
use std::{sync::Arc, time::Duration};
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer, trace::TraceLayer,
};

use crate::{
    core::{app_state::AppState, config::AppConfig, error::AppResult},
    infra::diesel::connection::init_db_pool,
    modules::{
        ApiModule,
        health::controller::HealthModule,
        user::{controller::UserModule, repository::DbUserRepository, service::UserServiceImpl},
    },
};

pub fn build_router(state: AppState, config: &AppConfig) -> Router {
    Router::new()
        .nest("/api/v1", HealthModule::routes())
        .nest("/api/v1", UserModule::routes())
        .layer(CompressionLayer::new())
        .layer(RequestBodyLimitLayer::new(
            config.server.body_limit_mb * 1024 * 1024,
        ))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(config.server.request_timeout_s),
        ))
        .layer(build_cors_layer(config))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

fn build_cors_layer(config: &AppConfig) -> CorsLayer {
    let origins: Vec<HeaderValue> = config
        .server
        .cors_allowed_origins
        .iter()
        .filter_map(|origin| origin.parse::<HeaderValue>().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
        ])
}

pub async fn build_state(config: &AppConfig) -> AppResult<AppState> {
    let db_pool = init_db_pool(&config.db).await?;

    let user_repo = Arc::new(DbUserRepository::new(db_pool.clone()));
    let user_service = Arc::new(UserServiceImpl::new(user_repo));

    Ok(AppState {
        db_pool,
        user_service,
    })
}
