use std::sync::OnceLock;

use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "OTEL_SDK_DISABLED", default = "false")]
    pub otel_sdk_disabled: bool,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn config() -> &'static Config {
    CONFIG.get_or_init(|| Config::init_from_env().unwrap())
}
