use crate::core::read_config::{read_all_api_definitions, read_all_origin_definitions};
use crate::core::reverse_proxy::{deploy_mgt_server, deploy_reverse_proxy};

mod configuration_reader;
mod core;
mod file_utils;

#[allow(unused_must_use)]
fn main() {
    read_all_api_definitions();
    read_all_origin_definitions();
    std::thread::spawn(move || {
        match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(deploy_mgt_server(8888))
        {
            Ok(_) => {
                println!("Deployed single threaded runtime!");
                println!("Deployed management server on single threaded runtime!");
            }
            Err(e) => {
                eprintln!("Error while deploying management server - {}", e);
                panic!();
            }
        }
    });
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(deploy_reverse_proxy(8080));
}
