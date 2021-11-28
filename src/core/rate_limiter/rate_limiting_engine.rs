use std::collections::HashMap;
use std::num::NonZeroU32;

use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use nonzero_ext::nonzero;
use tokio::sync::mpsc::Receiver;

use crate::configuration_reader::origin_def_reader::{RateLimiterConfig, TimeUnit};
use crate::RateLimiterAPI;

fn calculate_rps(rate_limiter_config: RateLimiterConfig) -> u32 {
    match rate_limiter_config.time_unit {
        TimeUnit::Second => rate_limiter_config.req_per_time_unit,
        TimeUnit::Minute => (rate_limiter_config.req_per_time_unit as f32 / 60.0f32).ceil() as u32,
        TimeUnit::Hour => (rate_limiter_config.req_per_time_unit as f32 / 3600.0f32).ceil() as u32,
    }
}

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
                RateLimiterAPI::UpdateOriginSpecification {
                    origin_id,
                    rate_limiter_spec,
                } => match NonZeroU32::new(calculate_rps(rate_limiter_spec)) {
                    None => {
                        api_rate_limiter_map.insert(
                            origin_id,
                            RateLimiter::direct(Quota::per_second(nonzero!(100u32))),
                        );
                    }
                    Some(calculated_rps) => {
                        api_rate_limiter_map.insert(
                            origin_id,
                            RateLimiter::direct(Quota::per_second(calculated_rps)),
                        );
                    }
                },
            }
        }
    }
}
