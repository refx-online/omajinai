use crate::{config::Config, error::AppError};

use anyhow::Result;
use refx_pp::Beatmap;

use std::{collections::HashMap, path::Path};
use tokio::{fs::read, fs::write, sync::RwLock};
use tracing::{info, warn};

pub struct BeatmapService {
    client: reqwest::Client,
    config: Config,
    cache: RwLock<HashMap<i32, Beatmap>>,
}

impl BeatmapService {
    pub fn new(client: reqwest::Client, config: Config) -> Self {
        Self {
            client,
            config,
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub async fn fetch_beatmap_osu_file(&self, beatmap_id: i32) -> Result<Vec<u8>, AppError> {
        let base_url = &self.config.beatmaps_service_url;
        let url = format!("{base_url}/v1/get-osu/{beatmap_id}");

        info!("fetching beatmap {} from {}", beatmap_id, url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::ExternalService(e.to_string()))?;

        if response.status() == 404 {
            return Err(AppError::BeatmapNotFound(beatmap_id));
        }

        let response = response
            .error_for_status()
            .map_err(|e| AppError::ExternalService(e.to_string()))?;

        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| AppError::ExternalService(e.to_string()))?
            .to_vec();

        Ok(response_bytes)
    }

    pub async fn get_beatmap(&self, beatmap_id: i32) -> Result<Beatmap, AppError> {
        {
            let cache = self.cache.read().await;
            if let Some(beatmap) = cache.get(&beatmap_id) {
                info!("beatmap {} found in cache", beatmap_id);
                return Ok(beatmap.clone());
            }
        }

        let beatmap_path = Path::new(&self.config.beatmaps_path).join(format!("{beatmap_id}.osu"));

        let bytes = match read(&beatmap_path).await {
            Ok(d) => {
                info!("beatmap {} loaded from disk", beatmap_id);
                d
            },
            Err(_) => {
                let f = self.fetch_beatmap_osu_file(beatmap_id).await?;
                write(&beatmap_path, &f).await.ok();
                info!("beatmap {} fetched", beatmap_id);
                f
            },
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
                warn!("cache size limit reached, removed old entries");
            }
        }

        info!("beatmap {} cached successfully", beatmap_id);
        Ok(beatmap)
    }
}
