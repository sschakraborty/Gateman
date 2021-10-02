use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};

async fn hello_world_resp_function(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new("Hello, World".into()))
}

async fn ctrl_c_shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}

#[tokio::main]
pub async fn deploy_reverse_proxy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let reverse_proxy_server_address = SocketAddr::from(([127, 0, 0, 1], 3000));
    let make_svc_metadata = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(hello_world_resp_function))
    });

    let server = Server::bind(&reverse_proxy_server_address).serve(make_svc_metadata);
    let graceful = server.with_graceful_shutdown(ctrl_c_shutdown_signal());

    if let Err(e) = graceful.await {
        eprintln!("Server error: {}", e);
    }
    Ok(())
}
