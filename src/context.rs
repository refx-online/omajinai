use crate::{
    config::Config,
    services::{beatmap::BeatmapService, performance::PerformanceService},
};

use redis::Client as RedisClient;
use sqlx::{MySqlPool, mysql::MySqlPoolOptions};

use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

#[derive(Clone)]
pub struct Context {
    pub config: Config,
    pub http_client: reqwest::Client,
    pub beatmap_service: Arc<BeatmapService>,
    pub performance_service: Arc<PerformanceService>,
    pub redis: RedisClient,
    pub database: MySqlPool,
    pub start_time: SystemTime,
}

impl Context {
    pub async fn new(config: Config) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.request_timeout_seconds))
            .build()?;

        let beatmap_service = Arc::new(BeatmapService::new(http_client.clone(), config.clone()));

        let performance_service = Arc::new(PerformanceService::new(config.cache_size).await);

        let redis = RedisClient::open(config.redis_dsn.clone())?;
        let mut conn = redis.get_multiplexed_async_connection().await?;

        let _: () = redis::AsyncCommands::ping(&mut conn).await?;

        let database = MySqlPoolOptions::new().connect(&config.mysql_dsn).await?;

        Ok(Self {
            config,
            http_client,
            beatmap_service,
            performance_service,
            redis,
            database,
            start_time: SystemTime::now(),
        })
    }
}
