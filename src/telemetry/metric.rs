use std::sync::OnceLock;

use opentelemetry::{global, metrics::Histogram};
use opentelemetry_semantic_conventions::metric::HTTP_SERVER_REQUEST_DURATION;

/// Explicit bucket boundaries recommended by the HTTP semantic conventions for
/// `http.server.request.duration`.
///
/// [HTTP semantic conventions]: https://opentelemetry.io/docs/specs/semconv/http/http-metrics/#metric-httpserverrequestduration
const HTTP_SERVER_REQUEST_DURATION_BOUNDARIES: &[f64] = &[
    0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.75, 1.0, 2.5, 5.0, 7.5, 10.0,
];

/// The instruments defined by the
/// [HTTP server metrics](https://opentelemetry.io/docs/specs/semconv/http/http-metrics/#http-server)
/// semantic conventions.
pub struct Metrics {
    /// `http.server.request.duration` — duration of HTTP server requests, in seconds.
    pub http_server_request_duration: Histogram<f64>,
}

impl Metrics {
    fn new() -> Self {
        let meter = global::meter("revo");

        Self {
            http_server_request_duration: meter
                .f64_histogram(HTTP_SERVER_REQUEST_DURATION)
                .with_description("Duration of HTTP server requests.")
                .with_unit("s")
                .with_boundaries(HTTP_SERVER_REQUEST_DURATION_BOUNDARIES.to_vec())
                .build(),
        }
    }
}

static METRICS: OnceLock<Metrics> = OnceLock::new();

pub fn metrics() -> &'static Metrics {
    METRICS.get_or_init(Metrics::new)
}
