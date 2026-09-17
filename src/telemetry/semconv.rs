use std::{borrow::Cow, sync::OnceLock};

use axum::http::{HeaderMap, Method, StatusCode, Version, header};

use crate::core::config;

/// `http.request.method` value reported for methods the instrumentation does not know about.
pub const HTTP_REQUEST_METHOD_OTHER: &str = "_OTHER";

/// The HTTP methods considered "known" by the semantic conventions, in the order they are
/// listed there.
pub const DEFAULT_KNOWN_METHODS: &[&str] = &[
    "CONNECT", "DELETE", "GET", "HEAD", "OPTIONS", "PATCH", "POST", "PUT", "QUERY", "TRACE",
];

/// Longest `server.address` value that is still recorded.
///
/// The attribute is derived from a request header and is therefore attacker controlled, so
/// obviously bogus values are rejected instead of being turned into metric cardinality.
const MAX_SERVER_ADDRESS_LENGTH: usize = 253;

/// Returns the `http.request.method` attribute value for `method`.
///
/// The semantic conventions require methods that are not known to the instrumentation to be
/// reported as `_OTHER`. The set of known methods is a full replacement of
/// [`DEFAULT_KNOWN_METHODS`] and can be configured with the comma separated
/// `OTEL_INSTRUMENTATION_HTTP_KNOWN_METHODS` environment variable.
pub fn request_method(method: &Method) -> &'static str {
    known_methods()
        .iter()
        .find(|known| known.as_str() == method.as_str())
        .map(String::as_str)
        .unwrap_or(HTTP_REQUEST_METHOD_OTHER)
}

/// The methods configured through `OTEL_INSTRUMENTATION_HTTP_KNOWN_METHODS`, or
/// [`DEFAULT_KNOWN_METHODS`] when the variable is not set.
fn known_methods() -> &'static [String] {
    static KNOWN_METHODS: OnceLock<Vec<String>> = OnceLock::new();

    KNOWN_METHODS.get_or_init(|| match config().otel.http.known_methods.as_deref() {
        Some(methods) if !methods.trim().is_empty() => parse_known_methods(methods),
        _ => DEFAULT_KNOWN_METHODS
            .iter()
            .copied()
            .map(String::from)
            .collect(),
    })
}

/// Splits a comma separated list of case sensitive HTTP methods.
///
/// Unknown entries are kept as they are: an override that lists an invalid method simply
/// matches nothing.
fn parse_known_methods(methods: &str) -> Vec<String> {
    methods
        .split(',')
        .map(str::trim)
        .filter(|method| !method.is_empty())
        .map(String::from)
        .collect()
}

/// Returns the `url.scheme` attribute value for a request.
///
/// The scheme of the original request is taken from the `Forwarded#proto` or
/// `X-Forwarded-Proto` header when a reverse proxy provides one, and from the immediate peer
/// connection otherwise. revo does not terminate TLS itself, so the peer connection is
/// always `http`; behind a TLS terminating proxy the headers report `https`.
pub fn url_scheme(headers: &HeaderMap) -> Cow<'static, str> {
    let scheme = forwarded_value(headers, "proto")
        .or_else(|| header_list_value(headers, "x-forwarded-proto"))
        .map(unquote)
        .filter(|scheme| !scheme.is_empty());

    match scheme {
        Some("http") => Cow::Borrowed("http"),
        Some("https") => Cow::Borrowed("https"),
        Some(other) => Cow::Owned(other.to_ascii_lowercase()),
        None => Cow::Borrowed("http"),
    }
}

/// Returns the `server.address` and `server.port` attribute values for a request.
///
/// Both attributes are opt-in for metrics because they are derived from request headers a
/// client controls. The original host is read from `Forwarded#host` / `X-Forwarded-Host`
/// when present, and from `Host` otherwise; the port is read from the same value and falls
/// back to the default port of `scheme`.
pub fn server_address_port(headers: &HeaderMap, scheme: &str) -> Option<(String, u16)> {
    let host = forwarded_value(headers, "host")
        .or_else(|| header_list_value(headers, "x-forwarded-host"))
        .or_else(|| {
            headers
                .get(header::HOST)
                .and_then(|value| value.to_str().ok())
        })?;

    let (address, port) = split_host_port(unquote(host))?;
    if address.len() > MAX_SERVER_ADDRESS_LENGTH || address.chars().any(char::is_whitespace) {
        return None;
    }

    let port = port.unwrap_or(if scheme == "https" { 443 } else { 80 });

    Some((address, port))
}

/// Returns the `network.protocol.version` attribute value for a request.
///
/// The version is only reported when it is known to the instrumentation. The matching
/// `network.protocol.name` attribute is deliberately never set: the semantic conventions
/// only require it when the application layer is *not* HTTP.
pub fn protocol_version(version: Version) -> Option<&'static str> {
    match version {
        Version::HTTP_10 => Some("1.0"),
        Version::HTTP_11 => Some("1.1"),
        Version::HTTP_2 => Some("2"),
        Version::HTTP_3 => Some("3"),
        _ => None,
    }
}

/// Returns the `error.type` attribute value for a response, if the response is an error.
///
/// Per the [HTTP span status definition] a server request is only an error when the status
/// code is in the 5xx range; a 4xx response is a client side problem. The value is the
/// status code number, which keeps the attribute low cardinality.
///
/// [HTTP span status definition]: https://opentelemetry.io/docs/specs/semconv/http/http-spans/#status
pub fn error_type(status: StatusCode) -> Option<String> {
    status
        .is_server_error()
        .then(|| status.as_u16().to_string())
}

/// Reads a parameter of the `Forwarded` header, such as the `proto` in
/// `Forwarded: for=192.0.2.60;proto=https;by=203.0.113.43`.
///
/// The first matching parameter wins, which is the element contributed by the proxy closest
/// to the client. Parameter names are matched case insensitively.
fn forwarded_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get(header::FORWARDED)?
        .to_str()
        .ok()?
        .split([',', ';'])
        .filter_map(|parameter| parameter.split_once('='))
        .find(|(key, _)| key.trim().eq_ignore_ascii_case(name))
        .map(|(_, value)| value)
}

/// Reads the first element of a comma separated request header, such as
/// `X-Forwarded-Proto: https, http`.
fn header_list_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name)?.to_str().ok()?.split(',').next()
}

/// Removes the optional quotes of a `Forwarded` parameter value and trims surrounding
/// whitespace.
fn unquote(value: &str) -> &str {
    let value = value.trim();

    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
}

/// Splits a `host[:port]` authority into its parts, handling bracketed IPv6 literals.
fn split_host_port(host: &str) -> Option<(String, Option<u16>)> {
    if let Some(rest) = host.strip_prefix('[') {
        let (address, rest) = rest.split_once(']')?;
        let port = rest.strip_prefix(':').and_then(|port| port.parse().ok());

        return (!address.is_empty()).then(|| (address.to_owned(), port));
    }

    match host.rsplit_once(':') {
        Some((address, port)) if !address.is_empty() => {
            Some((address.to_owned(), port.parse().ok()))
        }
        _ if !host.is_empty() => Some((host.to_owned(), None)),
        _ => None,
    }
}
