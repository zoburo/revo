use std::sync::OnceLock;

use dotenvy::dotenv;

pub use config::{Config, config};

use crate::telemetry::provider::TelemetryProvider;

mod config;

static TELEMETRY_PROVIDER: OnceLock<TelemetryProvider> = OnceLock::new();

pub fn telemetry_provider() -> &'static TelemetryProvider {
    TELEMETRY_PROVIDER.get_or_init(TelemetryProvider::init)
}

pub fn init() {
    dotenv().ok();
    env_logger::init();

    let _ = config();

    if !config().otel_sdk_disabled {
        let _ = telemetry_provider();
    }
}

pub fn cleanup() {
    if !config().otel_sdk_disabled {
        telemetry_provider().shutdown();
    }
}
