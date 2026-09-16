use dotenvy::dotenv;

pub use config::{Config, config};

use crate::telemetry::provider::telemetry_provider;

mod config;
pub mod constant;

pub fn init() {
    dotenv().ok();
    env_logger::init();

    if !config().otel.sdk_disabled {
        let _ = telemetry_provider();
    }
}

pub fn cleanup() {
    if !config().otel.sdk_disabled {
        telemetry_provider().unwrap().shutdown();
    }
}
