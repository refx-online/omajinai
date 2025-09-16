use anyhow::Result;

use omajinai::{config::Config, context::Context, routes::create_routes};

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

    let (_addr, server) = warp::serve(routes).bind_with_graceful_shutdown(addr, shutdown);

    server.await;

    Ok(())
}
