use crate::{
    error::AppError,
    models::{requests::CalculateRequest, responses::PerformanceResult},
    services::beatmap::BeatmapService,
    utils::mods::{GameMods, parse_mods},
};
use refx_pp::Performance;
use refx_pp::model::mode::GameMode;

use std::sync::Arc;

pub struct PerformanceService;

impl PerformanceService {
    pub async fn new() -> Self {
        Self {}
    }

    pub async fn calculate_performance(
        &self,
        request: CalculateRequest,
        beatmap_service: Arc<BeatmapService>,
    ) -> Result<PerformanceResult, AppError> {
        request.validate()?;

        let beatmap = beatmap_service.get_beatmap(request.beatmap_id).await?;

        let mode = match request.mode {
            0 => GameMode::Osu,
            1 => GameMode::Taiko,
            2 => GameMode::Catch,
            3 => GameMode::Mania,
            _ => return Err(AppError::InvalidGameMode(request.mode)),
        };

        let mut calculator = beatmap
            .performance()
            .mode_or_ignore(mode)
            .lazer(request.lazer.unwrap_or(false))
            .accuracy(request.accuracy);

        if let Some(combo) = request.max_combo {
            calculator = calculator.combo(combo);
        }

        if let Some(misses) = request.miss_count {
            calculator = calculator.misses(misses);
        }

        if let Some(passed) = request.passed_objects {
            calculator = calculator.passed_objects(passed);
        }

        if let Some(legacy_score) = request.legacy_score {
            calculator = calculator.legacy_total_score(legacy_score);
        }

        if let Some(mods_str) = &request.mods {
            let mods = parse_mods(mods_str, mode).unwrap_or_default();
            calculator = match mods {
                GameMods::Legacy(legacy_mods) => calculator.mods(legacy_mods),
                GameMods::Intermode(intermode_mods) => calculator.mods(intermode_mods),
                GameMods::Lazer(lazer_mods) => calculator.mods(lazer_mods),
            };
        }
        // NOTE: we clone here before it gets consumed
        let hypothetical_calculator = calculator.clone();

        let result = calculator.calculate();
        let hypothetical_result = Self::with_misses(hypothetical_calculator, 0).calculate();

        let perf_result = PerformanceResult::from_attributes(result, hypothetical_result.pp());

        println!(
            "Calculated performance: {:.2}pp, {:.2}*",
            perf_result.pp, perf_result.stars
        );

        Ok(perf_result)
    }

    fn with_misses(calculator: Performance<'_>, misses: u32) -> Performance<'_> {
        calculator.misses(misses)
    }
}
