use axum::{body::Body, http::Request, middleware::Next, response::Response};
use opentelemetry::{KeyValue, trace::TraceContextExt};
use tracing::info_span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::telemetry::metrics;

pub async fn telemetry(req: Request<Body>, next: Next) -> Response {
    let method = req.method().to_string();
    let span = info_span!(
        "http.request",
        http.request.method = %method,
        url.path = %req.uri().path(),
    );
    let cx = span.context();
    let response = {
        let _guard = span.enter();
        next.run(req).await
    };

    metrics()
        .http_request
        .add(1, &[KeyValue::new("http.request.method", method)]);

    cx.span().add_event(
        "request.completed",
        vec![opentelemetry::KeyValue::new(
            "http.response.status_code",
            response.status().as_u16() as i64,
        )],
    );

    response
}
