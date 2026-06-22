use crate::{
    core::{
        app_state::AppState, config::AppConfig, error::AppResult, logging::log_request_middleware,
        request_context::request_context_middleware,
    },
    infra::{
        diesel::connection::init_db_pool,
        valkey::{connection::init_valkey_connection, token_store::RedisTokenStore},
    },
    modules::{
        auth::{
            controller::AuthModule,
            guard::{require_admin, require_auth},
            service::AuthServiceImpl,
            token::JwtService,
        },
        health::controller::HealthModule,
        user::{controller::UserModule, repository::DbUserRepository, service::UserServiceImpl},
    },
};
use axum::{
    Router,
    http::{HeaderValue, StatusCode},
    middleware,
};
use std::{sync::Arc, time::Duration};
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer,
};

// TODO: fix clone later
pub fn build_router(state: AppState, config: &AppConfig) -> Router {
    let public_routes = Router::new()
        .nest("/api/v1", HealthModule::routes())
        .nest("/api/v1", AuthModule::public_routes());

    let protected_routes = Router::new()
        .nest("/api/v1", AuthModule::protected_routes())
        .nest("/api/v1", UserModule::routes())
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    let admin_routes = Router::new()
        .nest("/api/v1", UserModule::admin_routes())
        .route_layer(middleware::from_fn_with_state(state.clone(), require_admin));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(admin_routes)
        .layer(CompressionLayer::new())
        .layer(RequestBodyLimitLayer::new(
            config.server.body_limit_mb * 1024 * 1024,
        ))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(config.server.request_timeout_s),
        ))
        .layer(build_cors_layer(config))
        .layer(middleware::from_fn(log_request_middleware))
        .layer(middleware::from_fn(request_context_middleware))
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
    let redis_client = init_valkey_connection(&config.valkey).await?;

    let jwt_service = JwtService::new(config.jwt.clone());

    let token_store = Arc::new(RedisTokenStore::new(redis_client));
    let user_repo = Arc::new(DbUserRepository::new(db_pool.clone())); // TODO: fix clone later

    let auth_service = Arc::new(AuthServiceImpl::new(
        user_repo.clone(),
        token_store,
        jwt_service,
    ));

    let user_service = Arc::new(UserServiceImpl::new(user_repo));

    Ok(AppState {
        db_pool,
        auth_service,
        user_service,
    })
}
