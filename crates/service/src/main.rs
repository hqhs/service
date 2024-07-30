use std::env;

use service::setup_app;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL")?;
    let app = setup_app(&database_url)?;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    tracing::error!("serving requests...");

    axum::serve(listener, app).await?;

    Ok(())
}
