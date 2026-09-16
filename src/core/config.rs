use std::sync::OnceLock;

use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "ENVIRONMENT", default = "local")]
    pub environment: String,

    #[envconfig(nested)]
    pub http: Http,

    #[envconfig(nested)]
    pub otel: Otel,
}

#[derive(Envconfig)]
pub struct Http {
    #[envconfig(from = "HTTP_ADDRESS", default = "0.0.0.0:3000")]
    pub address: String,
}

#[derive(Envconfig)]
pub struct Otel {
    #[envconfig(from = "OTEL_SDK_DISABLED", default = "true")]
    pub sdk_disabled: bool,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

/// Get config
pub fn config() -> &'static Config {
    CONFIG.get_or_init(|| Config::init_from_env().unwrap())
}
