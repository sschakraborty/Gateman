use crate::core::reverse_proxy::{deploy_mgt_server, deploy_reverse_proxy};

mod configuration_reader;
mod core;
mod file_utils;

#[allow(unused_must_use)]
fn main() {
    deploy_mgt_server(8888);
    deploy_reverse_proxy(8080);
}
