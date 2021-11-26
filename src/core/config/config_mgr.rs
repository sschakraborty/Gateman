use std::sync::Arc;

use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot::Sender;

use crate::ConfigMgrProxyAPI;
use crate::configuration_reader::api_def_reader::{APIDefinition, APISpecification};
use crate::core::config::read_config::{read_all_api_definitions, read_all_origin_definitions};

pub(crate) async fn deploy_config_mgr(mut receiver: Receiver<ConfigMgrProxyAPI>) {
    let api_definitions = Arc::new(read_all_api_definitions());
    let _origin_definitions = Arc::new(read_all_origin_definitions());
    while let api_call = receiver.recv().await {
        if api_call.is_some() {
            let api_call = api_call.unwrap();
            let api_definitions = api_definitions.clone();
            tokio::spawn(async move {
                match api_call {
                    ConfigMgrProxyAPI::GetAPIDefinitionBySpecification {
                        specification,
                        responder,
                    } => get_api_def_by_specification(specification, responder, api_definitions),
                    ConfigMgrProxyAPI::GetAPIDefinitionByID { api_id, responder } => {
                        get_api_def_by_id(api_id, responder, api_definitions)
                    }
                }
            });
        }
    }
}

fn get_api_def_by_specification(
    query_specification: APISpecification,
    responder: Sender<Option<APIDefinition>>,
    api_definitions: Arc<Vec<APIDefinition>>,
) {
    let mut matching_path_api_def_vec: Vec<&APIDefinition> = vec![];
    for api_definition in api_definitions.as_ref() {
        let api_specification = api_definition.specification_as_ref();
        for query_path in &query_specification.paths {
            for api_path in &api_specification.paths {
                if query_path.starts_with(api_path) {
                    matching_path_api_def_vec.push(api_definition);
                }
            }
        }
    }
    let mut matching_method_api_def_vec: Vec<&APIDefinition> = vec![];
    for api_definition in matching_path_api_def_vec {
        for method in &query_specification.methods {
            if (&api_definition.specification_as_ref().methods).contains(method) {
                matching_method_api_def_vec.push(api_definition);
            }
        }
    }
    let mut matching_hostname_api_def_vec = vec![];
    for api_definition in matching_method_api_def_vec {
        for hostname in &query_specification.hostnames {
            if (&api_definition.specification_as_ref().hostnames).contains(hostname) {
                matching_hostname_api_def_vec.push(api_definition);
            }
        }
    }
    if matching_hostname_api_def_vec.is_empty() {
        responder.send(Option::None);
    } else {
        // Todo: Implement the proper responder
        responder.send(Option::None);
    }
}

fn get_api_def_by_id(
    _api_id: String,
    responder: Sender<Option<APIDefinition>>,
    _api_definitions: Arc<Vec<APIDefinition>>,
) {
    responder.send(Option::None);
}
