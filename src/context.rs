use crate::{
    config::Config,
    services::{beatmap::BeatmapService, performance::PerformanceService},
};

use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

#[derive(Clone)]
pub struct Context {
    pub config: Config,
    pub http_client: reqwest::Client,
    pub beatmap_service: Arc<BeatmapService>,
    pub performance_service: Arc<PerformanceService>,
    pub start_time: SystemTime,
}

impl Context {
    pub async fn new(config: Config) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.request_timeout_seconds))
            .build()?;

        let beatmap_service = Arc::new(BeatmapService::new(http_client.clone(), config.clone()));

        let performance_service = Arc::new(PerformanceService::new().await);

        Ok(Self {
            config,
            http_client,
            beatmap_service,
            performance_service,
            start_time: SystemTime::now(),
        })
    }
}
