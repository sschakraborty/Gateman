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
            let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(32);
            tokio::join!(
                tokio::spawn(deploy_mgt_server(8888)),
                tokio::spawn(deploy_reverse_proxy(8080))
            );
        });
}
