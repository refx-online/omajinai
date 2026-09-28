use crate::{
    context::Context,
    models::responses::{ApiResponse, HealthResponse},
};
use std::{convert::Infallible, sync::Arc};
use warp::Reply;

pub async fn health_handler(_context: Arc<Context>) -> Result<impl Reply, Infallible> {
    // NOTE: elapsed() errs on clock step (NTP) — report zero uptime instead
    // of panicking the health endpoint.
    let uptime = _context.start_time.elapsed().unwrap_or_default().as_secs();

    let response = ApiResponse::success(HealthResponse {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        uptime_seconds: uptime,
    });

    Ok(warp::reply::json(&response))
}
