use std::convert::Infallible;

use hyper::header::{HeaderValue, CONTENT_ENCODING, CONTENT_TYPE};
use hyper::{Body, Method, Request, Response, StatusCode};
use tokio::sync::mpsc::Sender;

use crate::configuration_reader::api_def_reader::{APIDefinition, APISpecification};
use crate::configuration_reader::origin_def_reader::Origin;
use crate::core::config::config_mgr_proxy_api::ConfigMgrProxyAPI;
use crate::ConfigMgrProxyAPI::{GetAPIDefinitionBySpecification, GetOriginDefinitionByID};

fn create_404_not_found_response() -> Result<Response<Body>, Infallible> {
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

fn create_503_service_unavailable_response() -> Result<Response<Body>, Infallible> {
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

fn create_500_int_error_response() -> Result<Response<Body>, Infallible> {
    let response = Response::new("Error While Processing".into());
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

pub async fn route_mgt_server(
    request: Request<Body>,
    _sender: Sender<ConfigMgrProxyAPI>,
) -> Result<Response<Body>, Infallible> {
    match (request.method(), request.uri().path()) {
        (&Method::GET, "/status") => {
            let response = Response::new("{\n    \"status\": \"healthy\"\n}".into());
            let (mut parts, body) = response.into_parts();
            parts
                .headers
                .append(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            Ok(Response::from_parts(parts, body))
        }
        (_, _) => create_404_not_found_response(),
    }
}

pub async fn route_proxy_server(
    request: Request<Body>,
    sender: Sender<ConfigMgrProxyAPI>,
) -> Result<Response<Body>, Infallible> {
    let (responder, receiver) = tokio::sync::oneshot::channel();
    let find_api_call = GetAPIDefinitionBySpecification {
        specification: APISpecification {
            methods: vec![request.method().to_string()],
            paths: vec![request.uri().path().to_string()],
            hostnames: vec![String::from(
                request
                    .headers()
                    .get(hyper::http::header::HOST)
                    .unwrap()
                    .to_str()
                    .unwrap(),
            )],
        },
        responder,
    };
    sender.send(find_api_call).await;
    let response = receiver.await;
    match response {
        Ok(result) => match result {
            None => create_404_not_found_response(),
            Some(api_definition) => {
                let (responder, receiver) = tokio::sync::oneshot::channel();
                let find_origin_call = GetOriginDefinitionByID {
                    origin_id: api_definition.origin_id(),
                    responder,
                };
                sender.send(find_origin_call).await;
                let response = receiver.await;
                match response {
                    Ok(result) => match result {
                        None => create_503_service_unavailable_response(),
                        Some(origin_definition) => {
                            process_request_to_origin(api_definition, origin_definition, request)
                        }
                    },
                    Err(_) => create_500_int_error_response(),
                }
            }
        },
        Err(_) => create_500_int_error_response(),
    }
}

async fn process_request_to_origin(
    _api_definition: APIDefinition,
    _origin_definition: Origin,
    _request: Request<Body>,
) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Found API and origin defs!")))
}
