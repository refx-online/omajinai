use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CalculateRequest {
    pub beatmap_id: Option<i32>,
    pub mode: u32,
    pub mods: Option<String>,
    pub max_combo: Option<u32>,
    pub accuracy: f64,
    pub miss_count: Option<u32>,
    pub passed_objects: Option<u32>,
    pub lazer: Option<bool>,
}

impl CalculateRequest {
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        if self.beatmap_id.is_none() {
            return Err(crate::error::AppError::BadRequest(
                "beatmap_id must be provided".to_string(),
            ));
        }

        if self.mode > 3 {
            return Err(crate::error::AppError::InvalidGameMode(self.mode));
        }

        if self.accuracy < 0.0 || self.accuracy > 100.0 {
            return Err(crate::error::AppError::InvalidAccuracy(self.accuracy));
        }

        Ok(())
    }
}
