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

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::DatabasePool(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Configuration(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::AddressParse(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            AppError::Unauthorized => "UNAUTHORIZED",
            AppError::Forbidden => "FORBIDDEN",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Conflict(_) => "CONFLICT",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::Database(_) => "INTERNAL_ERROR",
            AppError::DatabasePool(_) => "INTERNAL_ERROR",
            AppError::Redis(_) => "INTERNAL_ERROR",
            AppError::Internal => "INTERNAL_ERROR",
            AppError::Configuration(_) => "INTERNAL_ERROR",
            AppError::AddressParse(_) => "INTERNAL_ERROR",
            AppError::Io(_) => "INTERNAL_ERROR",
        }
    }

    pub fn client_message(&self) -> String {
        match self {
            AppError::Unauthorized => "Authentication is required.".to_string(),
            AppError::Forbidden => "You do not have permission to perform this action.".to_string(),
            AppError::NotFound(message) => {
                if message.trim().is_empty() {
                    "Resource not found.".to_string()
                } else {
                    message.clone()
                }
            },
            AppError::Conflict(message) => {
                if message.trim().is_empty() {
                    "Resource already exists.".to_string()
                } else {
                    message.clone()
                }
            },
            AppError::Validation(message) => {
                if message.trim().is_empty() {
                    "Invalid request data.".to_string()
                } else {
                    message.clone()
                }
            },
            AppError::Database(_)
            | AppError::Configuration(_)
            | AppError::AddressParse(_)
            | AppError::Io(_)
            | AppError::DatabasePool(_)
            | AppError::Redis(_)
            | AppError::Internal => "Something went wrong. Please try again later.".to_string(),
        }
    }

    pub fn should_log_as_error(&self) -> bool {
        self.status_code().is_server_error()
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();

        let body = ErrorResponse {
            code: self.code().to_string(),
            message: self.client_message(),
        };

        (status, Json(body)).into_response()
    }
}
