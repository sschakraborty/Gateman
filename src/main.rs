use crate::core::reverse_proxy::deploy_reverse_proxy;

mod configuration_reader;
mod core;
mod file_utils;

fn main() {
    deploy_reverse_proxy();
}
