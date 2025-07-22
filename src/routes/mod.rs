use crate::{
    context::Context,
    handlers::{health, performance},
    models::requests::{BulkCalculateRequest, CalculateRequest},
};
use std::sync::Arc;
use warp::Filter;

pub fn create_routes(
    context: Arc<Context>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "authorization"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"]);

    let context_filter = warp::any().map(move || context.clone());

    let health_route = warp::path("health")
        .and(warp::get())
        .and(context_filter.clone())
        .and_then(health::health_handler);

    let calculate_route = warp::path("calculate")
        .and(warp::post())
        .and(warp::body::json::<CalculateRequest>())
        .and(context_filter.clone())
        .and_then(performance::calculate_handler);

    let bulk_calculate_route = warp::path("bulk-calculate")
        .and(warp::post())
        .and(warp::body::json::<BulkCalculateRequest>())
        .and(context_filter.clone())
        .and_then(performance::bulk_calculate_handler);

    health_route
        .or(calculate_route)
        .or(bulk_calculate_route)
        .with(cors)
        .with(warp::log("omajinai"))
}