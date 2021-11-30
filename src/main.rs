use log::info;
use tokio::sync::mpsc;

use crate::core::config::config_mgr::deploy_config_mgr;
use crate::core::config::config_mgr_proxy_api::ConfigMgrProxyAPI;
use crate::core::rate_limiter::rate_limiter_api::RateLimiterAPI;
use crate::core::rate_limiter::rate_limiting_engine::deploy_rate_limiter;
use crate::core::reverse_proxy::{deploy_mgt_server, deploy_reverse_proxy};
use crate::utils::path_utils::get_directory_of_executable;

mod configuration_reader;
mod core;
mod file_utils;
mod utils;

#[allow(unused_must_use)]
fn main() {
    log4rs::init_file(
        get_directory_of_executable().join("resources/config/logging.yml"),
        Default::default(),
    )
        .unwrap();
    info!("Starting executor runtime");
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let (rate_limiter_tx, rate_limiter_rx) = mpsc::channel::<RateLimiterAPI>(32);
            let (config_mgr_tx, config_mgr_rx) = mpsc::channel::<ConfigMgrProxyAPI>(32);
            tokio::select!(
                _ = tokio::spawn(deploy_rate_limiter(rate_limiter_rx)) => 0,
                _ = tokio::spawn(deploy_config_mgr(config_mgr_rx, rate_limiter_tx.clone())) => 0,
                _ = tokio::spawn(deploy_mgt_server(8888, config_mgr_tx.clone())) => 0,
                _ = tokio::spawn(deploy_reverse_proxy(
                    8080,
                    config_mgr_tx.clone(),
                    rate_limiter_tx.clone()
                )) => 0
            );
        });
}
