use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel_async::pooled_connection::{PoolError, bb8::RunError};
use serde::Serialize;
use thiserror::Error;

pub type AppResult<T = ()> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("not found: {0}")]
    NotFound(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("database error")]
    Database(DieselError),

    #[error("database pool error: {0}")]
    DatabasePool(String),

    #[error("redis error")]
    Redis(#[from] redis::RedisError),

    #[error("configuration error: {0}")]
    Configuration(#[from] anyhow::Error),

    #[error("invalid server address: {0}")]
    AddressParse(#[from] std::net::AddrParseError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("internal server error")]
    Internal,
}

pub fn map_diesel_error(error: DieselError) -> AppError {
    match error {
        DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
            AppError::Conflict("resource already exists".to_string())
        },
        other => AppError::Database(other),
    }
}

impl From<DieselError> for AppError {
    fn from(error: DieselError) -> Self {
        map_diesel_error(error)
    }
}

impl From<PoolError> for AppError {
    fn from(error: PoolError) -> Self {
        AppError::DatabasePool(error.to_string())
    }
}

impl From<RunError> for AppError {
    fn from(error: RunError) -> Self {
        AppError::DatabasePool(error.to_string())
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "unauthorized".to_string(),
            ),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN", "forbidden".to_string()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg),
            AppError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                "database error".to_string(),
            ),
            AppError::DatabasePool(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_POOL_ERROR",
                "database pool error".to_string(),
            ),
            AppError::Redis(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "REDIS_ERROR",
                "redis error".to_string(),
            ),
            AppError::Configuration(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "CONFIGURATION_ERROR",
                error.to_string(),
            ),
            AppError::AddressParse(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ADDRESS_PARSE_ERROR",
                "invalid server address".to_string(),
            ),
            AppError::Io(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "IO_ERROR",
                error.to_string(),
            ),
            AppError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "internal server error".to_string(),
            ),
        };

        (
            status,
            Json(ErrorResponse {
                code: code.to_string(),
                message,
            }),
        )
            .into_response()
    }
}
