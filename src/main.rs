use std::env;
use std::sync::Arc;
use std::net::SocketAddr;

use axum::routing::{Router, get};
use sqlx::sqlite::SqlitePoolOptions;
use axum::response::IntoResponse;
use axum::extract::State;
// use axum::ServiceExt;
use hyper::StatusCode;
// use diesel::sqlite::SqliteConnection;
// use diesel::prelude::*;
use dotenvy::dotenv;
// use tokio::sync::Mutex;
// use tokio::signal;
// use tokio::prelude::*;

// pub mod models;
// pub mod schema;

// use self::models::Post;
// use self::schema::posts::dsl::*;

struct ServerState {
}

type StateTy = State<Arc<ServerState>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&database_url).await?;

    let row: (i64, ) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&pool).await?;


    let state = Arc::new(ServerState{});

    let app = Router::new()
        .route("/posts", get(list_posts))
        .fallback(handler_404)
        .with_state(state);

    // let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    // axum::Server::bind(&addr)
    //     .serve(app.into_make_service())
    //     .await
    //     .unwrap();

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}

async fn list_posts(State(_state): StateTy) -> impl IntoResponse {
    (StatusCode::OK, "Not implemented")
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404! Nothing to see here!")
}
