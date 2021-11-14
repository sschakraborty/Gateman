use tokio::sync::mpsc::Receiver;

use crate::ConfigMgrProxyAPI;

pub(crate) async fn deploy_config_mgr(receiver: Receiver<ConfigMgrProxyAPI>) {}
