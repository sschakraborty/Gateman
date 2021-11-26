use tokio::sync::mpsc;

use crate::core::config::config_mgr::deploy_config_mgr;
use crate::core::config::config_mgr_proxy_api::ConfigMgrProxyAPI;
use crate::core::reverse_proxy::{deploy_mgt_server, deploy_reverse_proxy};

mod configuration_reader;
mod core;
mod file_utils;

#[allow(unused_must_use)]
fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let (tx, rx) = mpsc::channel::<ConfigMgrProxyAPI>(32);
            tokio::join!(
                tokio::spawn(deploy_config_mgr(rx)),
                tokio::spawn(deploy_mgt_server(8888, tx.clone())),
                tokio::spawn(deploy_reverse_proxy(8080, tx.clone()))
            );
        });
}
