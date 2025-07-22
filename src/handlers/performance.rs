use crate::{
    context::Context,
    models::{
        requests::{BulkCalculateRequest, CalculateRequest},
        responses::{ApiResponse, BulkCalculateResponse},
    },
};
use std::{convert::Infallible, sync::Arc};
use tracing::error;
use warp::Reply;

pub async fn calculate_handler(
    request: CalculateRequest,
    context: Arc<Context>,
) -> Result<warp::reply::Response, Infallible>{
    match context
        .performance_service
        .calculate_performance(request, context.beatmap_service.clone())
        .await
    {
        Ok(result) => {
            let response = ApiResponse::success(result);
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                warp::http::StatusCode::OK,
            ).into_response())
        }
        Err(err) => {
            error!("performance calculation failed: {}", err);
            Ok(err.to_response().into_response())
        }
    }
}

pub async fn bulk_calculate_handler(
    request: BulkCalculateRequest,
    context: Arc<Context>,
) -> Result<impl Reply, Infallible> {
    let mut results = Vec::new();
    let mut successful = 0;
    
    for calc_request in request.calculations {
        match context
            .performance_service
            .calculate_performance(calc_request, context.beatmap_service.clone())
            .await
        {
            Ok(result) => {
                results.push(Ok(result));
                successful += 1;
            }
            Err(err) => {
                results.push(Err(err.to_string()));
            }
        }
    }
    
    let response = ApiResponse::success(BulkCalculateResponse {
        processed: results.len(),
        successful,
        results,
    });
    
    Ok(warp::reply::json(&response))
}