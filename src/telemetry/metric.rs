use std::sync::OnceLock;

use opentelemetry::global;
use opentelemetry::metrics::Counter;

pub struct Metrics {
    pub http_request: Counter<u64>,
}

impl Metrics {
    fn new() -> Self {
        let meter = global::meter("revo");

        Self {
            http_request: meter
                .u64_counter("http.requests")
                .with_description("The number of requests")
                .build(),
        }
    }
}

static METRICS: OnceLock<Metrics> = OnceLock::new();

pub fn metrics() -> &'static Metrics {
    METRICS.get_or_init(Metrics::new)
}
