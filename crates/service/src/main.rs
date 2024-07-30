use std::env;

use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use service::setup_app;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber =
        FmtSubscriber::builder().with_max_level(Level::INFO).finish();
    tracing::subscriber::set_global_default(subscriber)?;

    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL")?;
    let app = setup_app(&database_url)?;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    tracing::info!("serving requests...");

    axum::serve(listener, app).await?;

    Ok(())
}
