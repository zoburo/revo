use anyhow::Result;
use axum::{Json, Router, middleware::from_fn, routing::get};
use serde_json::json;
use tokio::net::TcpListener;

use crate::{core::config, http::server::middleware::telemetry};

mod middleware;

pub struct HttpServer;

impl HttpServer {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(&self) -> Result<()> {
        let address = &config().http.address;
        let listener = TcpListener::bind(address).await?;
        let mut router =
            Router::new().route("/", get(async || Json(json!({"message": "Hello, world!"}))));

        if !config().otel.sdk_disabled {
            router = router.layer(from_fn(telemetry));
        }

        log::info!("Running http server on {address}");

        axum::serve(listener, router).await?;

        Ok(())
    }
}
