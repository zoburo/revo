use std::time::Instant;

use axum::{body::Body, extract::MatchedPath, http::Request, middleware::Next, response::Response};
use opentelemetry::{KeyValue, trace::TraceContextExt};
use opentelemetry_semantic_conventions::attribute::{
    ERROR_TYPE, HTTP_REQUEST_METHOD, HTTP_RESPONSE_STATUS_CODE, HTTP_ROUTE,
    NETWORK_PROTOCOL_VERSION, SERVER_ADDRESS, SERVER_PORT, URL_SCHEME,
};
use tracing::info_span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::{
    core::config,
    telemetry::{metrics, semconv},
};

pub async fn telemetry(req: Request<Body>, next: Next) -> Response {
    let started_at = Instant::now();

    // Everything that lives on the request has to be read before the inner service consumes
    // it.
    let method = semconv::request_method(req.method());
    let scheme = semconv::url_scheme(req.headers());
    let protocol_version = semconv::protocol_version(req.version());
    let route = req
        .extensions()
        .get::<MatchedPath>()
        .map(|path| path.as_str().to_owned());
    let server = if config().otel.http.capture_server_attributes {
        semconv::server_address_port(req.headers(), &scheme)
    } else {
        None
    };

    let metrics = metrics();

    // The attributes that are already known while the request runs. They are the complete
    // attribute set of `http.server.active_requests` and the shared prefix of the attribute
    // sets recorded when the request finishes.
    let mut attributes = vec![
        KeyValue::new(HTTP_REQUEST_METHOD, method),
        KeyValue::new(URL_SCHEME, scheme),
    ];
    if let Some((address, port)) = server {
        attributes.push(KeyValue::new(SERVER_ADDRESS, address));
        attributes.push(KeyValue::new(SERVER_PORT, i64::from(port)));
    }

    // The span keeps reporting the method as it was received, metrics report the method as
    // the semantic conventions define it.
    let span = info_span!(
        "http.request",
        http.request.method = %req.method(),
        url.path = %req.uri().path(),
    );
    let cx = span.context();
    let response = {
        let _guard = span.enter();
        next.run(req).await
    };

    let status = response.status();
    if let Some(route) = route {
        attributes.push(KeyValue::new(HTTP_ROUTE, route));
    }
    if let Some(version) = protocol_version {
        attributes.push(KeyValue::new(NETWORK_PROTOCOL_VERSION, version));
    }
    attributes.push(KeyValue::new(
        HTTP_RESPONSE_STATUS_CODE,
        i64::from(status.as_u16()),
    ));
    if let Some(error_type) = semconv::error_type(status) {
        attributes.push(KeyValue::new(ERROR_TYPE, error_type));
    }

    metrics
        .http_server_request_duration
        .record(started_at.elapsed().as_secs_f64(), &attributes);

    cx.span().add_event(
        "request.completed",
        vec![KeyValue::new(
            "http.response.status_code",
            i64::from(status.as_u16()),
        )],
    );

    response
}
