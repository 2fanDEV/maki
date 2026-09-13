#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = maki::config::load()?;
    env_logger::Builder::new()
        .parse_filters(&config.logging.rust_log)
        .parse_write_style(&config.logging.rust_log_style)
        .init();
    let database = maki::database::connect(
        &config.database.mongodb_uri,
        &config.database.mongodb_database,
    )
    .await?;
    let app = maki::api::router(database);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}
