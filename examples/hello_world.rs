use anyhow::Result;
use revo::http::server::HttpServer;

#[tokio::main]
async fn main() -> Result<()> {
    revo::core::init();

    let http_server = HttpServer::new();

    http_server.run().await?;

    revo::core::cleanup();

    Ok(())
}
