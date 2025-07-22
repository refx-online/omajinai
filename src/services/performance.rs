use crate::{
    error::AppError,
    models::{requests::CalculateRequest, responses::PerformanceResult},
    services::beatmap::BeatmapService,
    utils::mods::{GameMods, parse_mods},
};
use refx_pp::{model::mode::GameMode};

use std::sync::Arc;
use tracing::info;

pub struct PerformanceService {
    // TODO: caching?
}

impl PerformanceService {
    pub async fn new(_cache_size: usize) -> Self {
        Self {}
    }
    
    pub async fn calculate_performance(
        &self,
        request: CalculateRequest,
        beatmap_service: Arc<BeatmapService>,
    ) -> Result<PerformanceResult, AppError> {
        request.validate()?;
        
        let beatmap = if let Some(beatmap_id) = request.beatmap_id {
            beatmap_service.get_beatmap(beatmap_id).await?
        } else {
            return Err(AppError::BadRequest(
                "beatmap_id must be provided".to_string(),
            ));
        };
        
        let mode = match request.mode {
            0 => GameMode::Osu,
            1 => GameMode::Taiko,
            2 => GameMode::Catch,
            3 => GameMode::Mania,
            _ => return Err(AppError::InvalidGameMode(request.mode)),
        };
        
        let mut calculator = beatmap
            .performance()
            .try_mode(mode)
            .map_err(|_| AppError::Internal("failed to set game mode".to_string()))?
            .lazer(request.lazer.unwrap_or(false))
            .accuracy(request.accuracy);
        
        if let Some(combo) = request.max_combo {
            calculator = calculator.combo(combo);
        }
        
        if let Some(misses) = request.miss_count {
            calculator = calculator.misses(misses);
        }
        /*
        if let Some(passed) = request.passed_objects {
            calculator = calculator.passed_objects(passed);
        }
        */
        
        if let Some(mods_str) = &request.mods {
            let mods = parse_mods(mods_str, mode).unwrap_or_default();
            calculator = match mods {
                GameMods::Legacy(legacy_mods) => {
                    if request.lazer.unwrap_or(false) {
                        calculator.mods(legacy_mods)
                    } else {
                        calculator.mods(legacy_mods.bits())
                    }
                },
                GameMods::Intermode(intermode_mods) => calculator.mods(intermode_mods),
                GameMods::Lazer(lazer_mods) => calculator.mods(lazer_mods),
            };
        }
        
        let result = calculator.calculate();
        let perf_result = PerformanceResult::from_attributes(result);
        
        info!("calculated performance: {:.2}pp, {:.2}*", perf_result.pp, perf_result.stars);
        
        Ok(perf_result)
    }
}