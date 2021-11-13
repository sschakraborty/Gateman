use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};

use crate::core::router::{route_at_top_level, route_mgt_server};

async fn ctrl_c_shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}

#[tokio::main]
pub async fn deploy_mgt_server(port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let frontend_server_address = SocketAddr::from(([127, 0, 0, 1], port));
    let make_svc_metadata =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(route_mgt_server)) });

    let server = Server::bind(&frontend_server_address).serve(make_svc_metadata);
    let graceful = server.with_graceful_shutdown(ctrl_c_shutdown_signal());

    if let Err(e) = graceful.await {
        eprintln!("Server error: {}", e);
    }
    Ok(())
}

#[tokio::main]
pub async fn deploy_reverse_proxy(
    port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let frontend_server_address = SocketAddr::from(([127, 0, 0, 1], port));
    let make_svc_metadata =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(route_at_top_level)) });

    let server = Server::bind(&frontend_server_address).serve(make_svc_metadata);
    let graceful = server.with_graceful_shutdown(ctrl_c_shutdown_signal());

    if let Err(e) = graceful.await {
        eprintln!("Server error: {}", e);
    }
    Ok(())
}
