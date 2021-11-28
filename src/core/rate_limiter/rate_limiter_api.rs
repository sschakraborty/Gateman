use tokio::sync::oneshot::Sender;

use crate::configuration_reader::origin_def_reader::RateLimiterConfig;

pub enum RateLimiterAPI {
    ShouldProgress {
        origin_id: String,
        responder: Sender<Result<(), ()>>,
    },
    UpdateOriginSpecification {
        origin_id: String,
        rate_limiter_spec: RateLimiterConfig,
    },
}
