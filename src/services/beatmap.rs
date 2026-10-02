use crate::{config::Config, error::AppError};

use anyhow::Result;
use refx_pp::Beatmap;

use std::{collections::HashMap, path::Path};
use tokio::{fs, sync::RwLock};

pub struct BeatmapService {
    config: Config,
    cache: RwLock<HashMap<i32, Beatmap>>,
    http: reqwest::Client,
}

impl BeatmapService {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            cache: RwLock::new(HashMap::new()),
            http: reqwest::Client::new(),
        }
    }

    pub async fn get_beatmap(&self, beatmap_id: i32) -> Result<Beatmap, AppError> {
        {
            let cache = self.cache.read().await;
            if let Some(beatmap) = cache.get(&beatmap_id) {
                return Ok(beatmap.clone());
            }
        }

        let beatmap_path = Path::new(&self.config.beatmaps_path).join(format!("{beatmap_id}.osu"));

        // Local files can rot (truncated downloads, bad writes). The parser
        // is lenient: garbage parses to zero objects and would silently
        // yield 0pp forever. Anything unparseable _or_ empty is deleted so
        // it gets refetched below instead of poisoning pp calculation.
        if let Ok(bytes) = fs::read(&beatmap_path).await {
            match parse_valid(&bytes) {
                Some(beatmap) => {
                    self.insert_cache(beatmap_id, beatmap.clone()).await;
                    return Ok(beatmap);
                }
                None => {
                    eprintln!(
                        "corrupt beatmap file {beatmap_id}, refetching from mirror"
                    );
                    let _ = fs::remove_file(&beatmap_path).await;
                }
            }
        }

        let mirror = self.config.beatmap_service_url.as_deref()
            .ok_or(AppError::BeatmapNotFound(beatmap_id))?;

        let url = format!("{mirror}/v1/get-osu/{beatmap_id}");
        let resp = self.http.get(&url).send().await
            .map_err(|e| AppError::ExternalService(format!("mirror fetch failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(AppError::BeatmapNotFound(beatmap_id));
        }

        let data = resp.bytes().await
            .map_err(|e| AppError::ExternalService(format!("mirror read failed: {e}")))?
            .to_vec();

        // Validate before persisting so a corrupt download never lands on disk.
        let beatmap = parse_valid(&data).ok_or_else(|| {
            AppError::ExternalService(format!(
                "mirror returned unparseable beatmap {beatmap_id}"
            ))
        })?;

        if let Some(parent) = beatmap_path.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| AppError::Internal(format!("failed to create beatmap dir: {e}")))?;
        }
        fs::write(&beatmap_path, &data).await
            .map_err(|e| AppError::Internal(format!("failed to save beatmap: {e}")))?;

        self.insert_cache(beatmap_id, beatmap.clone()).await;

        Ok(beatmap)
    }

    async fn insert_cache(&self, beatmap_id: i32, beatmap: Beatmap) {
        let mut cache = self.cache.write().await;
        cache.insert(beatmap_id, beatmap);

        if cache.len() > self.config.cache_size {
            let keys_to_remove: Vec<i32> = cache
                .keys()
                .take(cache.len() - self.config.cache_size)
                .cloned()
                .collect();
            for key in keys_to_remove {
                cache.remove(&key);
            }
        }
    }
}

/// Parse that also rejects degenerate maps: a file with zero hit objects is
/// either corrupt or useless for pp, so callers treat it as missing.
fn parse_valid(bytes: &[u8]) -> Option<Beatmap> {
    match Beatmap::from_bytes(bytes) {
        Ok(beatmap) if !beatmap.hit_objects.is_empty() => Some(beatmap),
        _ => None,
    }
}
