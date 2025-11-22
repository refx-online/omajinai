use serde::Serialize;
use std::{error::Error, fmt};
use warp::{Reply, http::StatusCode};

#[derive(Debug)]
pub enum AppError {
    BeatmapNotFound(i32),
    InvalidGameMode(u32),
    InvalidAccuracy(f64),
    ExternalService(String),
    Internal(String),
    BadRequest(String),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    pub code: u16,
}

impl warp::reject::Reject for AppError {}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::BeatmapNotFound(id) => write!(f, "Beatmap not found: {id}"),

            AppError::InvalidGameMode(mode) => write!(f, "Invalid game mode: {mode}"),

            AppError::InvalidAccuracy(acc) => write!(f, "Invalid accuracy: {acc}"),

            AppError::ExternalService(msg) => write!(f, "External service error: {msg}"),

            AppError::Internal(msg) => write!(f, "Internal error: {msg}"),

            AppError::BadRequest(msg) => write!(f, "Bad request: {msg}"),
        }
    }
}

impl Error for AppError {}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::BeatmapNotFound(_) => StatusCode::NOT_FOUND,
            AppError::InvalidGameMode(_) => StatusCode::BAD_REQUEST,
            AppError::InvalidAccuracy(_) => StatusCode::BAD_REQUEST,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::ExternalService(_) => StatusCode::BAD_GATEWAY,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn to_response(&self) -> impl Reply {
        let code = self.status_code();
        let response = ErrorResponse {
            success: false,
            error: self.to_string(),
            code: code.as_u16(),
        };

        warp::reply::with_status(warp::reply::json(&response), code)
    }
}
