use crate::{
    error::AppError,
    models::{requests::CalculateRequest, responses::PerformanceResult},
    services::beatmap::BeatmapService,
    utils::mods::{GameMods, parse_mods},
};
use refx_pp::model::mode::GameMode;

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::Arc,
};
use tokio::sync::RwLock;

/// unique.
unsafe fn hash(request: &CalculateRequest) -> u64 {
    use std::collections::hash_map::DefaultHasher;

    let mut hasher = DefaultHasher::new();

    request.beatmap_id.hash(&mut hasher);
    request.mode.hash(&mut hasher);
    request.mods.hash(&mut hasher);
    request.lazer.hash(&mut hasher);

    // hash accuracy as bits to avoid float comparison issues
    #[allow(unnecessary_transmutes)]
    std::mem::transmute::<f64, u64>(request.accuracy).hash(&mut hasher);

    request.max_combo.hash(&mut hasher);
    request.miss_count.hash(&mut hasher);
    request.passed_objects.hash(&mut hasher);
    request.legacy_score.hash(&mut hasher);

    hasher.finish()
}

pub struct PerformanceService {
    cache: RwLock<HashMap<u64, PerformanceResult>>,
    cache_size: usize,
}

impl PerformanceService {
    pub async fn new(cache_size: usize) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            cache_size,
        }
    }

    pub async fn calculate_performance(
        &self,
        request: CalculateRequest,
        beatmap_service: Arc<BeatmapService>,
    ) -> Result<PerformanceResult, AppError> {
        request.validate()?;

        let key = unsafe { hash(&request) };

        {
            let c = self.cache.read().await;
            if let Some(result) = c.get(&key) {
                return Ok(result.clone());
            }
        }

        let beatmap = if let Some(beatmap_id) = request.beatmap_id {
            beatmap_service.get_beatmap(beatmap_id).await?
        } else {
            return Err(AppError::BadRequest(
                "Beatmap_id must be provided".to_string(),
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

        if let Some(legacy_score) = request.legacy_score {
            calculator = calculator.legacy_total_score(legacy_score);
        }

        let result = calculator.calculate();
        let perf_result = PerformanceResult::from_attributes(result);

        println!(
            "Calculated performance: {:.2}pp, {:.2}*",
            perf_result.pp, perf_result.stars
        );

        {
            let mut cache = self.cache.write().await;
            cache.insert(key, perf_result.clone());

            // at the time when im writing this,
            // i don't think caching is a good idea for this case
            // since rosu-pp is already fast enough
            if cache.len() > self.cache_size {
                let keys_to_remove: Vec<u64> = cache
                    .keys()
                    .take(cache.len() - self.cache_size)
                    .cloned()
                    .collect();
                for key in keys_to_remove {
                    cache.remove(&key);
                }
            }
        }

        Ok(perf_result)
    }
}
