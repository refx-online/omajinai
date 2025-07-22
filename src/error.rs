use serde::Serialize;
use thiserror::Error;
use warp::{http::StatusCode, Reply};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("beatmap not found: {0}")]
    BeatmapNotFound(i32),
    
    #[error("invalid game mode: {0}")]
    InvalidGameMode(u32),
    
    #[error("invalid accuracy: {0}")]
    InvalidAccuracy(f64),
    
    #[error("external service error: {0}")]
    ExternalService(String),
    
    #[error("internal error: {0}")]
    Internal(String),
    
    #[error("bad request: {0}")]
    BadRequest(String),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    pub code: u16,
}

impl warp::reject::Reject for AppError {}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::BeatmapNotFound(_)     => StatusCode::NOT_FOUND,
            AppError::InvalidGameMode(_)     => StatusCode::BAD_REQUEST,
            AppError::InvalidAccuracy(_)     => StatusCode::BAD_REQUEST,
            AppError::BadRequest(_)          => StatusCode::BAD_REQUEST,
            AppError::ExternalService(_)     => StatusCode::BAD_GATEWAY,
            AppError::Internal(_)            => StatusCode::INTERNAL_SERVER_ERROR,
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