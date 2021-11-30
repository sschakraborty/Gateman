use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot::Sender;

use crate::configuration_reader::api_def_reader::{APIDefinition, APISpecification};
use crate::configuration_reader::origin_def_reader::Origin;
use crate::core::config::read_config::{read_all_api_definitions, read_all_origin_definitions};
use crate::{ConfigMgrProxyAPI, RateLimiterAPI};

async fn send_origin_definitions_to_rate_limiter(
    rate_limiter_tx: tokio::sync::mpsc::Sender<RateLimiterAPI>,
    origin_definitions: &Vec<Origin>,
) {
    for origin_def in origin_definitions {
        rate_limiter_tx
            .send(RateLimiterAPI::UpdateOriginSpecification {
                origin_id: origin_def.origin_id.clone(),
                rate_limiter_spec: origin_def.specification.rate_limiter.clone(),
            })
            .await;
    }
}

async fn initialize(
    rate_limiter_tx: tokio::sync::mpsc::Sender<RateLimiterAPI>,
) -> (HashMap<String, APIDefinition>, HashMap<String, Origin>) {
    let api_definitions = read_all_api_definitions();
    let origin_definitions = read_all_origin_definitions();
    let mut api_def_map = HashMap::new();
    let mut origin_def_map = HashMap::new();
    send_origin_definitions_to_rate_limiter(rate_limiter_tx, &origin_definitions).await;
    for api_def in api_definitions {
        api_def_map.insert(api_def.api_id.clone(), api_def);
    }
    for origin_def in origin_definitions {
        origin_def_map.insert(origin_def.origin_id.clone(), origin_def);
    }
    (api_def_map, origin_def_map)
}

fn get_api_def_by_specification(
    query_specification: APISpecification,
    responder: Sender<Option<APIDefinition>>,
    api_definitions: Arc<HashMap<String, APIDefinition>>,
) {
    let mut matching_path_api_def_vec: Vec<&APIDefinition> = vec![];
    for api_definition in api_definitions.as_ref() {
        let api_definition = api_definition.1;
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

fn get_origin_def_by_id(
    origin_id: String,
    responder: Sender<Option<Origin>>,
    origin_definitions: Arc<HashMap<String, Origin>>,
) {
    responder.send(
        origin_definitions
            .as_ref()
            .get(&*origin_id)
            .map(|origin| origin.clone()),
    );
}

pub(crate) async fn deploy_config_mgr(
    mut receiver: Receiver<ConfigMgrProxyAPI>,
    rate_limiter_tx: tokio::sync::mpsc::Sender<RateLimiterAPI>,
) {
    let (api_definitions, origin_definitions) = initialize(rate_limiter_tx).await;
    let api_definitions = Arc::new(api_definitions);
    let origin_definitions = Arc::new(origin_definitions);
    loop {
        let api_call = receiver.recv().await;
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
                    ConfigMgrProxyAPI::GetOriginDefinitionByID {
                        origin_id,
                        responder,
                    } => get_origin_def_by_id(origin_id, responder, origin_definitions),
                }
            });
        }
    }
}
