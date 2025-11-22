use crate::{
    config::Config,
    services::{beatmap::BeatmapService, performance::PerformanceService},
};

use anyhow::Result;
use std::sync::Arc;
use std::time::SystemTime;

#[derive(Clone)]
pub struct Context {
    pub config: Config,
    pub beatmap_service: Arc<BeatmapService>,
    pub performance_service: Arc<PerformanceService>,
    pub start_time: SystemTime,
}

impl Context {
    pub async fn new(config: Config) -> Result<Self> {
        let beatmap_service = Arc::new(BeatmapService::new(config.clone()));

        let performance_service = Arc::new(PerformanceService::new().await);

        Ok(Self {
            config,
            beatmap_service,
            performance_service,
            start_time: SystemTime::now(),
        })
    }
}
