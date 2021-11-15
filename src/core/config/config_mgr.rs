use std::sync::Arc;

use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot::Sender;

use crate::configuration_reader::api_def_reader::{APIDefinition, APISpecification};
use crate::core::config::read_config::{read_all_api_definitions, read_all_origin_definitions};
use crate::ConfigMgrProxyAPI;

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
    _specification: APISpecification,
    responder: Sender<Option<APIDefinition>>,
    _api_definitions: Arc<Vec<APIDefinition>>,
) {
    responder.send(Option::None);
}

fn get_api_def_by_id(
    _api_id: String,
    responder: Sender<Option<APIDefinition>>,
    _api_definitions: Arc<Vec<APIDefinition>>,
) {
    responder.send(Option::None);
}
