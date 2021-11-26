use std::convert::Infallible;
use std::time::Duration;

use hyper::header::{HeaderValue, CONTENT_ENCODING, CONTENT_TYPE};
use hyper::{Body, Client, Method, Request, Response, StatusCode, Uri};
use rand::Rng;
use tokio::sync::mpsc::Sender;
use tokio::time::timeout;

use crate::configuration_reader::api_def_reader::{APIDefinition, APISpecification};
use crate::configuration_reader::origin_def_reader::{Origin, Server};
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

fn select_server(servers: &Vec<Server>) -> Option<&Server> {
    servers.get(rand::thread_rng().gen_range(0..servers.len()))
}

async fn process_request_to_origin(
    _api_definition: APIDefinition,
    origin_definition: Origin,
    request: Request<Body>,
) -> Result<Response<Body>, Infallible> {
    let client = Client::new();
    let mut req_to_origin = Request::from(request);
    let origin_spec = origin_definition.get_specification_ref();
    let server = select_server(&origin_spec.servers);
    match server {
        None => create_503_service_unavailable_response(),
        Some(server) => {
            let mut url_path = String::from("http://");
            url_path.push_str(server.hostname.as_str());
            url_path.push_str(":");
            url_path.push_str(server.port.to_string().as_str());
            url_path.push_str(req_to_origin.uri().path_and_query().unwrap().as_str());
            let uri_parse_result = url_path.as_str().parse::<Uri>();
            match uri_parse_result {
                Err(_) => create_500_int_error_response(),
                Ok(uri) => {
                    *req_to_origin.uri_mut() = uri;
                    let timeout_result =
                        timeout(Duration::from_millis(2500), client.request(req_to_origin)).await;
                    match timeout_result {
                        Err(_) => create_503_service_unavailable_response(),
                        Ok(origin_response) => match origin_response {
                            Err(_) => create_503_service_unavailable_response(),
                            Ok(response) => Ok(response),
                        },
                    }
                }
            }
        }
    }
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
                                .await
                        }
                    },
                    Err(_) => create_500_int_error_response(),
                }
            }
        },
        Err(_) => create_500_int_error_response(),
    }
}
