use std::sync::Arc;

use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot::Sender;

use crate::configuration_reader::api_def_reader::{APIDefinition, APISpecification};
use crate::configuration_reader::origin_def_reader::Origin;
use crate::core::config::read_config::{read_all_api_definitions, read_all_origin_definitions};
use crate::ConfigMgrProxyAPI;

pub(crate) async fn deploy_config_mgr(mut receiver: Receiver<ConfigMgrProxyAPI>) {
    let api_definitions = Arc::new(read_all_api_definitions());
    let origin_definitions = Arc::new(read_all_origin_definitions());
    while let api_call = receiver.recv().await {
        if api_call.is_some() {
            let api_call = api_call.unwrap();
            let api_definitions = api_definitions.clone();
            let origin_definitions = origin_definitions.clone();
            tokio::spawn(async move {
                match api_call {
                    ConfigMgrProxyAPI::GetAPIDefinitionBySpecification {
                        specification,
                        responder,
                    } => get_api_def_by_specification(specification, responder, api_definitions),
                    ConfigMgrProxyAPI::GetAPIDefinitionByID { api_id, responder } => {
                        get_api_def_by_id(api_id, responder, api_definitions)
                    }
                    ConfigMgrProxyAPI::GetOriginDefinitionByID {
                        origin_id,
                        responder,
                    } => get_origin_def_by_id(origin_id, responder, origin_definitions),
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
        let api_specification = &(api_definition.specification);
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
            if (&(api_definition.specification).methods).contains(method) {
                matching_method_api_def_vec.push(api_definition);
            }
        }
    }
    let mut matching_hostname_api_def_vec = vec![];
    for api_definition in matching_method_api_def_vec {
        for hostname in &query_specification.hostnames {
            if (&(api_definition.specification).hostnames).contains(hostname) {
                matching_hostname_api_def_vec.push(api_definition);
            }
        }
    }
    if matching_hostname_api_def_vec.is_empty() {
        responder.send(Option::None);
    } else {
        responder.send(
            matching_hostname_api_def_vec
                .pop()
                .map(|def_ref| def_ref.clone()),
        );
    }
}

fn get_api_def_by_id(
    _api_id: String,
    responder: Sender<Option<APIDefinition>>,
    _api_definitions: Arc<Vec<APIDefinition>>,
) {
    responder.send(Option::None);
}

fn get_origin_def_by_id(
    origin_id: String,
    responder: Sender<Option<Origin>>,
    origin_definitions: Arc<Vec<Origin>>,
) {
    let mut origin_def_selected: Option<Origin> = Option::None;
    let id_ref_to_find = &origin_id;
    for origin_definition in origin_definitions.as_ref() {
        if origin_definition.has_id(id_ref_to_find) {
            origin_def_selected = Option::Some(origin_definition.clone());
        }
    }
    responder.send(origin_def_selected);
}
