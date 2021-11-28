use std::collections::HashMap;
use std::num::NonZeroU32;

use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use nonzero_ext::nonzero;
use tokio::sync::mpsc::Receiver;

use crate::configuration_reader::origin_def_reader::{RateLimiterConfig, TimeUnit};
use crate::RateLimiterAPI;

fn create_non_zero_u32_from_u32(input: u32) -> NonZeroU32 {
    match NonZeroU32::new(input) {
        None => {
            nonzero!(100u32)
        }
        Some(parsed) => parsed,
    }
}

fn calculate_quota(rate_limiter_config: RateLimiterConfig) -> Quota {
    match rate_limiter_config.time_unit {
        TimeUnit::Hour => Quota::per_hour(create_non_zero_u32_from_u32(
            rate_limiter_config.req_per_time_unit,
        )),
        TimeUnit::Minute => Quota::per_minute(create_non_zero_u32_from_u32(
            rate_limiter_config.req_per_time_unit,
        )),
        TimeUnit::Second => Quota::per_second(create_non_zero_u32_from_u32(
            rate_limiter_config.req_per_time_unit,
        )),
    }
}

pub(crate) async fn deploy_rate_limiter(mut receiver: Receiver<RateLimiterAPI>) {
    let mut api_rate_limiter_map =
        HashMap::<String, RateLimiter<NotKeyed, InMemoryState, DefaultClock>>::new();
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
                } => {
                    api_rate_limiter_map.insert(
                        origin_id,
                        RateLimiter::direct(calculate_quota(rate_limiter_spec)),
                    );
                }
            }
        }
    }
}