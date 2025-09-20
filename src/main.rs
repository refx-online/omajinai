use anyhow::Result;

use omajinai::{
    config::Config, context::Context, routes::create_routes, services::recalculate::PubSubHandler,
};

use redis::AsyncCommands;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::from_env()?;
    info!("Loaded configuration: {:?}", config);

    let context = Arc::new(Context::new(config).await?);

    let pubsub = PubSubHandler::new(context.clone());
    tokio::spawn(async move {
        let _ = pubsub.start_listener().await;
    });

    // Tell bancho omajinai is up
    {
        let mut conn = context.redis_conn.lock().await;
        let _: () = conn.publish("refx:status", "0").await?;
    }

    let routes = create_routes(context.clone());

    let addr = ([0, 0, 0, 0], context.config.port);
    info!(
        "Performance service starting on http://{}",
        format!("{}:{}", addr.0.map(|x| x.to_string()).join("."), addr.1)
    );

    let shutdown = async {
        tokio::signal::ctrl_c().await.expect(""); // what even happened
        info!("Shutting down...");
    };

    let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(addr, shutdown);

    server.await;

    // Tell bancho omajinai is dead
    {
        let mut conn = context.redis_conn.lock().await;
        let _: () = conn.publish("refx:status", "1").await?;
    }

    Ok(())
}
