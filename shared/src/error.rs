use thiserror::Error;

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