use std::net::SocketAddr;

use tokio::net::TcpListener;

use service::setup_app;

#[tokio::test]
async fn test_home_page() -> anyhow::Result<()> {
    let app = setup_app("/tmp/test_db.sqlite3")?;
    let listener =
        TcpListener::bind("127.0.0.1:0".parse::<SocketAddr>()?).await?;
    let addr = listener.local_addr()?;
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let resp = reqwest::get(format!("http://{}", addr)).await?;

    assert_eq!(resp.status(), 200);

    Ok(())
}
