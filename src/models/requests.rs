use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CalculateRequest {
    pub beatmap_id: i32,
    pub mode: u32,
    pub mods: Option<String>,
    pub max_combo: Option<u32>,
    pub accuracy: f64,
    pub miss_count: Option<u32>,
    pub passed_objects: Option<u32>,
    pub legacy_score: Option<i64>,
    pub lazer: Option<bool>,
}

impl CalculateRequest {
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        if self.accuracy < 0.0 || self.accuracy > 100.0 {
            return Err(crate::error::AppError::InvalidAccuracy(self.accuracy));
        }

        if self.legacy_score.is_some() && self.lazer.unwrap_or(false) {
            return Err(crate::error::AppError::BadRequest(
                "Legacy score cannot be used with lazer calculations.".into(),
            ));
        }

        Ok(())
    }
}
