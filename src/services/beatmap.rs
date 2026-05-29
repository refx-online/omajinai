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

        let bytes = match fs::read(&beatmap_path).await {
            Ok(b) => b,
            Err(_) => {
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

                if let Some(parent) = beatmap_path.parent() {
                    fs::create_dir_all(parent).await
                        .map_err(|e| AppError::Internal(format!("failed to create beatmap dir: {e}")))?;
                }
                fs::write(&beatmap_path, &data).await
                    .map_err(|e| AppError::Internal(format!("failed to save beatmap: {e}")))?;

                data
            }
        };

        let beatmap = Beatmap::from_bytes(&bytes)
            .map_err(|e| AppError::Internal(format!("failed to parse beatmap: {e}")))?;

        {
            let mut cache = self.cache.write().await;
            cache.insert(beatmap_id, beatmap.clone());

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

        Ok(beatmap)
    }
}
