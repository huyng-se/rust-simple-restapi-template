use crate::{core::request_context::RequestContext, modules::auth::extractor::AuthContext};
use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

pub async fn log_request_middleware(req: Request, next: Next) -> Response {
    let started_at = Instant::now();

    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path().to_string();

    let request_id = req
        .extensions()
        .get::<RequestContext>()
        .map(|ctx| ctx.request_id.clone())
        .unwrap_or_else(|| "-".to_string());

    let user_id = req
        .extensions()
        .get::<AuthContext>()
        .map(|user| user.user_id.clone()); // TODO: fix clone later

    tracing::info!(
        request_id = %request_id,
        method = %method,
        path = %path,
        user_id = ?user_id,
        "request started"
    );

    let res = next.run(req).await;

    let status = res.status();
    let latency_ms = started_at.elapsed().as_millis() as u64;

    if status.is_server_error() {
        tracing::error!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = status.as_u16(),
            latency_ms,
            user_id = ?user_id,
            "request completed with server error"
        );
    } else if status.is_client_error() {
        tracing::warn!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = status.as_u16(),
            latency_ms,
            user_id = ?user_id,
            "request completed with client error"
        );
    } else {
        tracing::info!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = status.as_u16(),
            latency_ms,
            user_id = ?user_id,
            "request completed"
        );
    }

    res
}
