use anyhow::Result;

use omajinai::{
    config::Config, context::Context, routes::create_routes,
};

use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env()?;

    let context = Arc::new(Context::new(config).await?);

    let routes = create_routes(context.clone());

    let addr = ([0, 0, 0, 0], context.config.port);
    println!(
        "Performance service starting on http://{}",
        format!("{}:{}", addr.0.map(|x| x.to_string()).join("."), addr.1)
    );

    let shutdown = async {
        tokio::signal::ctrl_c().await.expect(""); // what even happened
        println!("Shutting down...");
    };

    let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(addr, shutdown);

    server.await;

    Ok(())
}
