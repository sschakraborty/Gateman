use tokio::sync::oneshot::Sender;

pub enum RateLimiterAPI {
    ShouldProgress {
        origin_id: String,
        responder: Sender<Result<(), ()>>,
    },
}
