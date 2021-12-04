use std::convert::Infallible;
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind};
use std::net::SocketAddr;
use std::sync::Arc;

use async_stream::stream;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use log::{debug, error, info};
use tokio::net::TcpListener;
use tokio::sync::mpsc::Sender;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
use tokio_rustls::TlsAcceptor;

use crate::core::router::route_proxy_server;
use crate::{ConfigMgrProxyAPI, RateLimiterAPI};

async fn ctrl_c_shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}

fn load_private_key(filename: &str) -> Result<PrivateKey, Error> {
    let key_file = File::open(filename)?;
    let mut reader = BufReader::new(key_file);
    let keys = rustls_pemfile::pkcs8_private_keys(&mut reader);
    match keys {
        Ok(keys) => {
            if keys.len() == 0 {
                return Err(Error::new(
                    ErrorKind::Other,
                    "Expecting one private key, found none",
                ));
            }
            if keys.len() != 1 {
                return Err(Error::new(
                    ErrorKind::Other,
                    "Expecting one private key, found more than one",
                ));
            }
            Ok(PrivateKey(keys[0].clone()))
        }
        Err(error) => {
            error!("Failed to read TLS private key - {}", error);
            Err(error)
        }
    }
}

fn load_certs(filename: &str) -> Result<Vec<Certificate>, Error> {
    let certificate_file = File::open(filename)?;
    let mut reader = BufReader::new(certificate_file);
    let certs = rustls_pemfile::certs(&mut reader);
    match certs {
        Ok(certs) => Ok(certs.into_iter().map(Certificate).collect()),
        Err(error) => {
            error!("Failed to read TLS certificates - {}", error);
            Err(error)
        }
    }
}

fn create_tls_config() -> Option<Arc<ServerConfig>> {
    match load_certs("resources/certs/proxy/certificate.crt") {
        Ok(certs) => match load_private_key("resources/certs/proxy/private.key") {
            Ok(key) => {
                let config = ServerConfig::builder()
                    .with_safe_defaults()
                    .with_no_client_auth()
                    .with_single_cert(certs, key);
                match config {
                    Ok(mut config) => {
                        config.alpn_protocols = vec![b"http/1.1".to_vec()];
                        Some(Arc::new(config))
                    }
                    Err(error) => {
                        error!("Could not read or validate TLS configuration - {}", error);
                        None
                    }
                }
            }
            Err(error) => {
                error!("Could not load private key - {}", error);
                None
            }
        },
        Err(error) => {
            error!("Could not load TLS certificate chain - {}", error);
            None
        }
    }
}

pub async fn deploy_tls_reverse_proxy(
    port: u16,
    config_mgr_tx: Sender<ConfigMgrProxyAPI>,
    rate_limiter_tx: Sender<RateLimiterAPI>,
) {
    info!("Deploying TLS reverse proxy server");
    match create_tls_config() {
        None => {
            error!("TLS configuration creation failed. Exiting TLS reverse proxy");
        }
        Some(tls_configuration) => {
            let frontend_server_address = SocketAddr::from(([127, 0, 0, 1], port));
            match TcpListener::bind(frontend_server_address).await {
                Ok(tcp) => {
                    let tls_acceptor = TlsAcceptor::from(tls_configuration);
                    let accept_stream = hyper::server::accept::from_stream(stream! {
                        use log::trace;
                        loop {
                            match tcp.accept().await {
                                Ok((socket, _)) => {
                                    let tls_accept_result = tls_acceptor.accept(socket).await;
                                    let tls_accept_result = tls_accept_result.map_err(|error| {
                                        trace!("Error during TLS handshake - {}", error);
                                        error
                                    });
                                    if tls_accept_result.is_ok() {
                                        yield tls_accept_result;
                                    }
                                }
                                Err(error) => {
                                    trace!("Error accepting TCP connection - {}", error);
                                }
                            }
                        }
                    });
                    let make_svc_metadata = make_service_fn(move |_| {
                        let rate_limiter_tx = rate_limiter_tx.clone();
                        let config_mgr_tx = config_mgr_tx.clone();
                        async move {
                            Ok::<_, Infallible>(service_fn(move |request| {
                                route_proxy_server(
                                    request,
                                    config_mgr_tx.clone(),
                                    rate_limiter_tx.clone(),
                                )
                            }))
                        }
                    });

                    let server = Server::builder(accept_stream).serve(make_svc_metadata);
                    let graceful = server.with_graceful_shutdown(ctrl_c_shutdown_signal());
                    let result = graceful.await;

                    if let Err(e) = result.as_ref() {
                        error!("TLS proxy server error: {}", e);
                    }
                }
                Err(error) => {
                    error!("TLS server could not be bound to port {} - {}", port, error);
                }
            }
        }
    }
    debug!("TLS reverse proxy server exited");
}
