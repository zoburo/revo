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

    /// Instrumentation options for HTTP servers.
    #[envconfig(nested)]
    pub http: OtelHttp,
}

#[derive(Envconfig)]
pub struct OtelHttp {
    /// Full replacement of the HTTP methods the instrumentation reports verbatim, comma
    /// separated and case sensitive. Every other method is reported as `_OTHER`.
    #[envconfig(from = "OTEL_INSTRUMENTATION_HTTP_KNOWN_METHODS")]
    pub known_methods: Option<String>,

    /// Captures the opt-in `server.address` and `server.port` metric attributes. They are
    /// derived from request headers, which clients control, so they are off by default.
    #[envconfig(
        from = "OTEL_INSTRUMENTATION_HTTP_SERVER_CAPTURE_SERVER_ATTRIBUTES",
        default = "false"
    )]
    pub capture_server_attributes: bool,

    /// Records the opt-in `http.server.request.body.size` and
    /// `http.server.response.body.size` metrics.
    #[envconfig(
        from = "OTEL_INSTRUMENTATION_HTTP_SERVER_CAPTURE_BODY_SIZE",
        default = "false"
    )]
    pub capture_body_size: bool,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

/// Get config
pub fn config() -> &'static Config {
    CONFIG.get_or_init(|| Config::init_from_env().unwrap())
}
