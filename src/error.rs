use thiserror::Error;
use actix_web::{HttpResponse, ResponseError};
use serde_json::json;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Matrix SDK error: {0}")]
    MatrixError(String),
    #[error("Invalid session ID")]
    InvalidSession,
    #[error("Not logged in")]
    NotLoggedIn,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Matrix SDK error: {0}")]
    MatrixSdk(#[from] matrix_sdk::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] matrix_sdk::HttpError),
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let status_code = match self {
            ApiError::InvalidSession => actix_web::http::StatusCode::BAD_REQUEST,
            ApiError::NotLoggedIn => actix_web::http::StatusCode::UNAUTHORIZED,
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        HttpResponse::build(status_code)
            .json(json!({
                "error": self.to_string()
            }))
    }
}