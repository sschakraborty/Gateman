use std::convert::Infallible;

use hyper::header::{HeaderValue, CONTENT_ENCODING, CONTENT_TYPE};
use hyper::http::response::Parts;
use hyper::{Body, Method, Request, Response, StatusCode, Uri};

pub async fn route_at_top_level(request: Request<Body>) -> Result<Response<Body>, Infallible> {
    match (request.method(), request.uri().path()) {
        (&Method::GET, "/status") => {
            let response = Response::new("{\n    \"status\": \"healthy\"\n}".into());
            let (mut parts, body) = response.into_parts();
            parts
                .headers
                .append(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            Ok(Response::from_parts(parts, body))
        }
        (_, _) => {
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
    }
}
