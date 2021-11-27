use std::collections::HashMap;

use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use nonzero_ext::nonzero;
use tokio::sync::mpsc::Receiver;

use crate::RateLimiterAPI;

pub(crate) async fn deploy_rate_limiter(mut receiver: Receiver<RateLimiterAPI>) {
    let mut api_rate_limiter_map =
        HashMap::<String, RateLimiter<NotKeyed, InMemoryState, DefaultClock>>::new();
    api_rate_limiter_map.insert(
        String::from("RFX829635"),
        RateLimiter::direct(Quota::per_second(nonzero!(5u32))),
    );
    loop {
        let api_call = receiver.recv().await;
        if api_call.is_some() {
            let api_call = api_call.unwrap();
            match api_call {
                RateLimiterAPI::ShouldProgress {
                    origin_id,
                    responder,
                } => match api_rate_limiter_map.get(&*origin_id) {
                    None => {
                        responder.send(Result::Err(()));
                    }
                    Some(rate_limiter) => {
                        responder.send(rate_limiter.check().map_err(|err| ()));
                    }
                },
            }
        }
    }
}
