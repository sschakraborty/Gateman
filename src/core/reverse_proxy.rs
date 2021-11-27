use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use tokio::sync::mpsc::Sender;

use crate::core::router::{route_mgt_server, route_proxy_server};
use crate::{ConfigMgrProxyAPI, RateLimiterAPI};

async fn ctrl_c_shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}

pub async fn deploy_mgt_server(
    port: u16,
    config_mgr_tx: Sender<ConfigMgrProxyAPI>,
) -> hyper::Result<()> {
    let frontend_server_address = SocketAddr::from(([127, 0, 0, 1], port));
    let make_svc_metadata = make_service_fn(move |_| {
        let config_mgr_tx = config_mgr_tx.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |request| {
                route_mgt_server(request, config_mgr_tx.clone())
            }))
        }
    });

    let server = Server::bind(&frontend_server_address).serve(make_svc_metadata);
    let graceful = server.with_graceful_shutdown(ctrl_c_shutdown_signal());
    let result = graceful.await;

    if let Err(e) = result.as_ref() {
        eprintln!("Management server error: {}", e);
    } else {
        println!("Management server started on port {}!", port);
    }
    result
}

pub async fn deploy_reverse_proxy(
    port: u16,
    config_mgr_tx: Sender<ConfigMgrProxyAPI>,
    rate_limiter_tx: Sender<RateLimiterAPI>,
) -> hyper::Result<()> {
    let frontend_server_address = SocketAddr::from(([127, 0, 0, 1], port));
    let make_svc_metadata = make_service_fn(move |_| {
        let rate_limiter_tx = rate_limiter_tx.clone();
        let config_mgr_tx = config_mgr_tx.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |request| {
                route_proxy_server(request, config_mgr_tx.clone(), rate_limiter_tx.clone())
            }))
        }
    });

    let server = Server::bind(&frontend_server_address).serve(make_svc_metadata);
    let graceful = server.with_graceful_shutdown(ctrl_c_shutdown_signal());
    let result = graceful.await;

    if let Err(e) = result.as_ref() {
        eprintln!("Proxy server error: {}", e);
    } else {
        println!("Proxy server started on port {}", port);
    }
    result
}
