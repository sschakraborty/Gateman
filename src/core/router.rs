use std::convert::Infallible;
use std::time::Duration;

use hyper::header::{HeaderValue, CONTENT_TYPE};
use hyper::{Body, Client, Method, Request, Response, Uri};
use rand::Rng;
use tokio::sync::mpsc::Sender;
use tokio::time::timeout;

use crate::configuration_reader::api_def_reader::{APIDefinition, APISpecification};
use crate::configuration_reader::origin_def_reader::{Origin, Server};
use crate::core::config::config_mgr_proxy_api::ConfigMgrProxyAPI;
use crate::core::rate_limiter::rate_limiter_api::RateLimiterAPI;
use crate::core::standard_response::{
    create_404_not_found_response, create_429_too_many_requests_response,
    create_500_int_error_response, create_503_service_unavailable_response,
    create_504_gateway_timeout_response,
};
use crate::ConfigMgrProxyAPI::{GetAPIDefinitionBySpecification, GetOriginDefinitionByID};
use crate::RateLimiterAPI::ShouldProgress;

fn select_server(servers: &Vec<Server>) -> Option<&Server> {
    servers.get(rand::thread_rng().gen_range(0..servers.len()))
}

async fn process_request_to_origin(
    rate_limiter_tx: Sender<RateLimiterAPI>,
    api_definition: APIDefinition,
    origin_definition: Origin,
    request: Request<Body>,
) -> Result<Response<Body>, Infallible> {
    let (responder, receiver) = tokio::sync::oneshot::channel();
    let rate_limit_check_call = ShouldProgress {
        origin_id: origin_definition.origin_id,
        responder,
    };
    rate_limiter_tx.send(rate_limit_check_call).await;
    let rate_limit_check_response = receiver.await;
    match rate_limit_check_response {
        Ok(rate_limit_check) => match rate_limit_check {
            Ok(_) => {
                let client = Client::new();
                let mut req_to_origin = Request::from(request);
                let origin_spec = &(origin_definition.specification);
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
                                let timeout_result = timeout(
                                    Duration::from_millis(api_definition.backend_response_timeout),
                                    client.request(req_to_origin),
                                )
                                .await;
                                match timeout_result {
                                    Err(_) => create_504_gateway_timeout_response(),
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
            Err(_) => create_429_too_many_requests_response(),
        },
        Err(_) => create_500_int_error_response(),
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
    config_mgr_tx: Sender<ConfigMgrProxyAPI>,
    rate_limiter_tx: Sender<RateLimiterAPI>,
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
    match config_mgr_tx.send(find_api_call).await {
        Err(_) => create_500_int_error_response(),
        Ok(_) => {
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
                        match config_mgr_tx.send(find_origin_call).await {
                            Err(_) => create_500_int_error_response(),
                            Ok(_) => {
                                let response = receiver.await;
                                match response {
                                    Ok(result) => match result {
                                        None => create_503_service_unavailable_response(),
                                        Some(origin_definition) => {
                                            process_request_to_origin(
                                                rate_limiter_tx,
                                                api_definition,
                                                origin_definition,
                                                request,
                                            )
                                            .await
                                        }
                                    },
                                    Err(_) => create_500_int_error_response(),
                                }
                            }
                        }
                    }
                },
                Err(_) => create_500_int_error_response(),
            }
        }
    }
}
