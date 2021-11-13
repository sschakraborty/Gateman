use crate::core::reverse_proxy::{deploy_mgt_server, deploy_reverse_proxy};

mod configuration_reader;
mod core;
mod file_utils;

fn main() {
    std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(deploy_mgt_server(8888));
    });
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(deploy_reverse_proxy(8080));
}
