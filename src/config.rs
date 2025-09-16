use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub port: u16,
    pub beatmaps_service_url: String,
    pub beatmaps_path: String,
    pub log_level: String,
    pub cache_size: usize,
    pub request_timeout_seconds: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 3030,
            beatmaps_service_url: "http://localhost:6969".to_string(),
            beatmaps_path: ".data/osu/".to_string(),
            log_level: "info".to_string(),
            cache_size: 1000,
            request_timeout_seconds: 30,
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let mut config = Config::default();

        if let Ok(port) = std::env::var("PORT") {
            config.port = port.parse()?;
        }

        if let Ok(beatmap_url) = std::env::var("BEATMAPS_SERVICE_URL") {
            config.beatmaps_service_url = beatmap_url;
        }

        if let Ok(beatmap_path) = std::env::var("BEATMAPS_PATH") {
            config.beatmaps_path = beatmap_path
        }

        if let Ok(log_level) = std::env::var("RUST_LOG") {
            config.log_level = log_level;
        }

        if let Ok(cache_size) = std::env::var("CACHE_SIZE") {
            config.cache_size = cache_size.parse()?;
        }

        if let Ok(timeout) = std::env::var("REQUEST_TIMEOUT_SECONDS") {
            config.request_timeout_seconds = timeout.parse()?;
        }

        Ok(config)
    }
}
