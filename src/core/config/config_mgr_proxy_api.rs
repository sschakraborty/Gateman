pub enum ConfigMgrProxyAPI {
    GetOrigin {
        key: String,
        // resp: Responder<Option<Bytes>>,
    },
    GetAPIDefinition {
        key: String,
        // val: Bytes,
        // resp: Responder<()>,
    },
}
