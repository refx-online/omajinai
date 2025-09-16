use crate::{
    context::Context,
    handlers::{health, performance},
    models::requests::CalculateRequest,
};
use std::sync::Arc;
use warp::Filter;

pub fn create_routes(
    context: Arc<Context>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let context_filter = warp::any().map(move || context.clone());

    let health_route = warp::path("health")
        .and(warp::get())
        .and(context_filter.clone())
        .and_then(health::health_handler);

    let calculate_route = warp::path("calculate")
        .and(warp::get())
        .and(warp::query::<CalculateRequest>())
        .and(context_filter.clone())
        .and_then(performance::calculate_handler);

    health_route.or(calculate_route).with(warp::log("omajinai"))
}
