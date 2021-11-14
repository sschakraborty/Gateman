use tokio::sync::oneshot::Sender;

use crate::configuration_reader::api_def_reader::{APIDefinition, APISpecification};

pub enum ConfigMgrProxyAPI {
    GetAPIDefinitionBySpecification {
        specification: APISpecification,
        responder: Sender<Option<APIDefinition>>,
    },
    GetAPIDefinitionByID {
        api_id: String,
        responder: Sender<Option<APIDefinition>>,
    },
}
