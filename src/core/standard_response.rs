use std::convert::Infallible;

use hyper::header::{HeaderValue, CONTENT_ENCODING, CONTENT_TYPE};
use hyper::{Body, Response, StatusCode};

pub(crate) fn create_404_not_found_response() -> Result<Response<Body>, Infallible> {
    let response = Response::new("404 Not Found".into());
    let (mut parts, body) = response.into_parts();
    parts.status = StatusCode::NOT_FOUND;
    parts.headers.append(
        CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    parts
        .headers
        .append(CONTENT_ENCODING, HeaderValue::from_static("utf-8"));
    Ok(Response::from_parts(parts, body))
}

pub(crate) fn create_503_service_unavailable_response() -> Result<Response<Body>, Infallible> {
    let response = Response::new("503 Service Unavailable".into());
    let (mut parts, body) = response.into_parts();
    parts.status = StatusCode::SERVICE_UNAVAILABLE;
    parts.headers.append(
        CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    parts
        .headers
        .append(CONTENT_ENCODING, HeaderValue::from_static("utf-8"));
    Ok(Response::from_parts(parts, body))
}

pub(crate) fn create_504_gateway_timeout_response() -> Result<Response<Body>, Infallible> {
    let response = Response::new("504 Gateway Timeout".into());
    let (mut parts, body) = response.into_parts();
    parts.status = StatusCode::GATEWAY_TIMEOUT;
    parts.headers.append(
        CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    parts
        .headers
        .append(CONTENT_ENCODING, HeaderValue::from_static("utf-8"));
    Ok(Response::from_parts(parts, body))
}

pub(crate) fn create_500_int_error_response() -> Result<Response<Body>, Infallible> {
    let response = Response::new("500 Internal Server Error".into());
    let (mut parts, body) = response.into_parts();
    parts.status = StatusCode::INTERNAL_SERVER_ERROR;
    parts.headers.append(
        CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    parts
        .headers
        .append(CONTENT_ENCODING, HeaderValue::from_static("utf-8"));
    Ok(Response::from_parts(parts, body))
}
