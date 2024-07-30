use std::{path::PathBuf, sync::Arc};

use models::Post;
use service::{add, build_root_router, ServerState};
use tera::Tera;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let _post = Post {
        id: Some(0),
        title: "Hello".to_string(),
        body: "World".to_string(),
    };

    let templates = {
        let path: PathBuf =
            [env!("CARGO_MANIFEST_DIR"), "templates"].iter().collect();
        if !path.is_dir() {
            anyhow::bail!(
                "{} directory does not exist",
                path.to_string_lossy()
            );
        }
        let glob = format!("{}/**/*.jinja2", path.to_string_lossy());
        let mut templates = Tera::new(&glob)?;
        templates.autoescape_on(vec![".jinja2"]);
        templates
    };

    let server = Arc::new(ServerState::new(templates));

    let static_path: PathBuf =
        [env!("CARGO_MANIFEST_DIR"), "static"].iter().collect();
    let app = build_root_router(server, static_path);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    println!("Hello world! {}", add(3, 4));

    axum::serve(listener, app).await?;

    Ok(())
}
