use tokio::sync::oneshot::Sender;

use crate::configuration_reader::api_def_reader::{APIDefinition, APISpecification};
use crate::configuration_reader::origin_def_reader::Origin;

pub enum ConfigMgrProxyAPI {
    GetAPIDefinitionBySpecification {
        specification: APISpecification,
        responder: Sender<Option<APIDefinition>>,
    },
    GetOriginDefinitionByID {
        origin_id: String,
        responder: Sender<Option<Origin>>,
    },
}
