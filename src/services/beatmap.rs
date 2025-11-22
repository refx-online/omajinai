use crate::{config::Config, error::AppError};

use anyhow::Result;
use refx_pp::Beatmap;

use std::{collections::HashMap, path::Path};
use tokio::{fs::read, sync::RwLock};

pub struct BeatmapService {
    config: Config,
    cache: RwLock<HashMap<i32, Beatmap>>,
}

impl BeatmapService {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            cache: RwLock::new(HashMap::new()),
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
        let bytes = read(&beatmap_path)
            .await
            .map_err(|_| AppError::BeatmapNotFound(beatmap_id))?;

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
