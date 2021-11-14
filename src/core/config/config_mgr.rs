use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot::Sender;

use crate::configuration_reader::api_def_reader::{APIDefinition, APISpecification};
use crate::ConfigMgrProxyAPI;

pub(crate) async fn deploy_config_mgr(mut receiver: Receiver<ConfigMgrProxyAPI>) {
    while let api_call = receiver.recv().await {
        if api_call.is_some() {
            let api_call = api_call.unwrap();
            tokio::spawn(async move {
                match api_call {
                    ConfigMgrProxyAPI::GetAPIDefinitionBySpecification {
                        specification,
                        responder,
                    } => get_api_def_by_specification(specification, responder),
                    ConfigMgrProxyAPI::GetAPIDefinitionByID { api_id, responder } => {
                        get_api_def_by_id(api_id, responder)
                    }
                }
            });
        }
    }
}

fn get_api_def_by_specification(
    _specification: APISpecification,
    responder: Sender<Option<APIDefinition>>,
) {
    responder.send(Option::None);
}

fn get_api_def_by_id(_api_id: String, responder: Sender<Option<APIDefinition>>) {
    responder.send(Option::None);
}
