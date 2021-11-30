use std::collections::HashMap;
use std::sync::Arc;

use log::{debug, info, trace};
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
    debug!(
        "Sending {} origin definitions to rate limiter",
        origin_definitions.len()
    );
    for origin_def in origin_definitions {
        trace!(
            "Sending origin definition (Origin ID: {}) to rate limiter",
            origin_def.origin_id
        );
        match rate_limiter_tx
            .send(RateLimiterAPI::UpdateOriginSpecification {
                origin_id: origin_def.origin_id.clone(),
                rate_limiter_spec: origin_def.specification.rate_limiter.clone(),
            })
            .await
        {
            Err(error) => {
                trace!(
                    "Failed to send origin definition (Origin ID: {}) to rate limiter as {}",
                    origin_def.origin_id,
                    error
                );
            }
            Ok(_) => {
                trace!(
                    "Sent origin definition (Origin ID: {}) to rate limiter",
                    origin_def.origin_id
                );
            }
        }
    }
}

async fn initialize(
    rate_limiter_tx: tokio::sync::mpsc::Sender<RateLimiterAPI>,
) -> (HashMap<String, APIDefinition>, HashMap<String, Origin>) {
    let api_definitions = read_all_api_definitions();
    debug!(
        "Configuration manager read {} api definitions",
        api_definitions.len()
    );

    let origin_definitions = read_all_origin_definitions();
    debug!(
        "Configuration manager read {} origin definitions",
        origin_definitions.len()
    );

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
        trace!("No APIDefinition matched the queried specification");
        match responder.send(Option::None) {
            Ok(_) => {
                trace!("Configuration manager responded successfully to query to get APIDefinition by specification")
            }
            Err(_) => {
                trace!("Configuration manager failed to respond to query to get APIDefinition by specification")
            }
        }
    } else {
        trace!("One or more APIDefinition(s) matched with the queried specification");
        match responder.send(
            matching_hostname_api_def_vec
                .pop()
                .map(|def_ref| def_ref.clone()),
        ) {
            Ok(_) => {
                trace!("Configuration manager responded successfully to query to get APIDefinition by specification")
            }
            Err(_) => {
                trace!("Configuration manager failed to respond to query to get APIDefinition by specification")
            }
        }
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
    info!("Deploying configuration manager");
    let (api_definitions, origin_definitions) = initialize(rate_limiter_tx).await;
    let api_definitions = Arc::new(api_definitions);
    let origin_definitions = Arc::new(origin_definitions);
    debug!(
        "Configuration manager read {} APIDefinition objects",
        api_definitions.as_ref().len()
    );
    debug!(
        "Configuration manager read {} Origin objects",
        origin_definitions.as_ref().len()
    );
    loop {
        let api_call = receiver.recv().await;
        if api_call.is_some() {
            trace!("Configuration manager received API call");
            let api_call = api_call.unwrap();
            let api_definitions = api_definitions.clone();
            let origin_definitions = origin_definitions.clone();
            tokio::spawn(async move {
                match api_call {
                    ConfigMgrProxyAPI::GetAPIDefinitionBySpecification {
                        specification,
                        responder,
                    } => {
                        trace!("Configuration manager received call for getting API definition by specification");
                        get_api_def_by_specification(specification, responder, api_definitions)
                    }
                    ConfigMgrProxyAPI::GetOriginDefinitionByID {
                        origin_id,
                        responder,
                    } => {
                        trace!("Configuration manager received call for getting Origin by ID");
                        get_origin_def_by_id(origin_id, responder, origin_definitions)
                    }
                }
            });
        }
    }
}
