use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub port: u16,
    pub beatmaps_path: String,
    pub cache_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 3030,
            beatmaps_path: ".data/osu/".into(),
            cache_size: 1000,
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let mut config = Config::default();

        if let Ok(port) = std::env::var("PORT") {
            config.port = port.parse()?;
        }

        if let Ok(beatmap_path) = std::env::var("BEATMAPS_PATH") {
            config.beatmaps_path = beatmap_path
        }

        if let Ok(cache_size) = std::env::var("CACHE_SIZE") {
            config.cache_size = cache_size.parse()?;
        }

        Ok(config)
    }
}
