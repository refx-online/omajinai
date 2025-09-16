use crate::{
    context::Context,
    models::{requests::CalculateRequest, responses::ApiResponse},
};
use std::{convert::Infallible, sync::Arc};
use tracing::error;
use warp::Reply;

pub async fn calculate_handler(
    request: CalculateRequest,
    context: Arc<Context>,
) -> Result<warp::reply::Response, Infallible> {
    match context
        .performance_service
        .calculate_performance(request, context.beatmap_service.clone())
        .await
    {
        Ok(result) => {
            let response = ApiResponse::success(result);
            Ok(
                warp::reply::with_status(warp::reply::json(&response), warp::http::StatusCode::OK)
                    .into_response(),
            )
        },
        Err(err) => {
            error!("performance calculation failed: {}", err);
            Ok(err.to_response().into_response())
        },
    }
}
