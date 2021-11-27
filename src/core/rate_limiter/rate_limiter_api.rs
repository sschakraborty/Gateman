use tokio::sync::oneshot::Sender;

pub(crate) enum RateLimiterAPI {
    ShouldProgress {
        origin_id: String,
        responder: Sender<Result<(), ()>>,
    },
}
