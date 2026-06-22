use axum::{
    extract::Request,
    http::header::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: String,
}

impl RequestContext {
    pub fn new() -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
        }
    }
}

pub static X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

pub async fn request_context_middleware(mut req: Request, next: Next) -> Response {
    let request_id = req
        .headers()
        .get(&X_REQUEST_ID)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string)
        .unwrap_or_else(|| RequestContext::new().request_id);

    let ctx = RequestContext {
        request_id: request_id.clone(),
    };

    req.extensions_mut().insert(ctx);

    let mut res = next.run(req).await;

    if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        res.headers_mut().insert(&X_REQUEST_ID, header_value);
    }

    res
}
