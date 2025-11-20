use thiserror::Error;
use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
};

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("internal server error: {0}")]
    InternalServer(String),

    #[error("database error: {0}")]
    Database(String),

}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::InternalServer(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        (status, message).into_response()
    }
}