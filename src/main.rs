#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let database = maki::database::from_env().await?;
    let app = maki::api::router(database);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}
