use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PerformanceResult {
    pub pp: f64,
    pub stars: f64,
    pub max_combo: u32,
}

impl PerformanceResult {
    pub fn from_attributes(attributes: refx_pp::any::PerformanceAttributes) -> Self {
        Self {
            pp: attributes.pp(),
            stars: attributes.stars(),
            max_combo: attributes.max_combo(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
}
