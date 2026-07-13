use crate::{core::request_context::RequestContext, modules::auth::extractor::AuthContext};
use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

pub async fn log_request_middleware(req: Request, next: Next) -> Response {
    let started_at = Instant::now();

    let request_id = req
        .extensions()
        .get::<RequestContext>()
        .map(|ctx| ctx.request_id.as_str())
        .unwrap_or("-");

    let user_id = req
        .extensions()
        .get::<AuthContext>()
        .map(|user| user.user_id.as_str())
        .unwrap_or("-");

    let span = tracing::info_span!(
        "http_request",
        request_id = %request_id,
        method = %req.method(),
        path = %req.uri().path(),
        user_id = %user_id,
    );

    span.in_scope(|| tracing::info!("request started"));

    let res = next.run(req).await;

    let status = res.status();
    let latency_ms = started_at.elapsed().as_millis() as u64;

    if status.is_server_error() {
        span.in_scope(|| {
            tracing::error!(
                status = status.as_u16(),
                latency_ms,
                "request completed with server error"
            )
        });
    } else if status.is_client_error() {
        span.in_scope(|| {
            tracing::warn!(
                status = status.as_u16(),
                latency_ms,
                "request completed with client error"
            )
        });
    } else {
        span.in_scope(|| tracing::info!(status = status.as_u16(), latency_ms, "request completed"));
    }

    res
}
