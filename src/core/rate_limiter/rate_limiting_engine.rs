use std::collections::HashMap;
use std::num::NonZeroU32;

use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use log::{debug, trace};
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
                        debug!(
                            "No rate limiter found for Origin (Origin ID: {})",
                            origin_id
                        );
                        match responder.send(Result::Err(())) {
                            Ok(_) => {
                                trace!("Rate limiter responded successfully for Origin (Origin ID: {})", origin_id)
                            }
                            Err(_) => {
                                trace!(
                                    "Rate limiter failed to respond for Origin (Origin ID: {})",
                                    origin_id
                                )
                            }
                        }
                    }
                    Some(rate_limiter) => {
                        match responder.send(rate_limiter.check().map_err(|_| ())) {
                            Ok(_) => {
                                trace!("Rate limiter responded successfully for Origin (Origin ID: {})", origin_id)
                            }
                            Err(_) => {
                                trace!(
                                    "Rate limiter failed to respond for Origin (Origin ID: {})",
                                    origin_id
                                )
                            }
                        }
                    }
                },
                RateLimiterAPI::UpdateOriginSpecification {
                    origin_id,
                    rate_limiter_spec,
                } => {
                    api_rate_limiter_map.insert(
                        origin_id.clone(),
                        RateLimiter::direct(calculate_quota(rate_limiter_spec)),
                    );
                    debug!(
                        "Origin specification updated in rate limiter for Origin (Origin ID: {})",
                        origin_id
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::num::NonZeroU32;

    use governor::Quota;

    use crate::configuration_reader::origin_def_reader::{RateLimiterConfig, TimeUnit};

    use super::{calculate_quota, create_non_zero_u32_from_u32};

    #[test]
    fn test_create_nonzero_u32_valid() {
        assert_eq!(
            NonZeroU32::new(500).unwrap(),
            create_non_zero_u32_from_u32(500)
        )
    }

    #[test]
    fn test_create_nonzero_u32_invalid() {
        assert_eq!(
            NonZeroU32::new(100).unwrap(),
            create_non_zero_u32_from_u32(0)
        )
    }

    #[test]
    fn test_calculate_quota_hour() {
        let rate_limiter_config = RateLimiterConfig {
            time_unit: TimeUnit::Hour,
            req_per_time_unit: 45,
        };
        assert_eq!(
            Quota::per_hour(NonZeroU32::new(45).unwrap()),
            calculate_quota(rate_limiter_config)
        );
    }

    #[test]
    fn test_calculate_quota_minute() {
        let rate_limiter_config = RateLimiterConfig {
            time_unit: TimeUnit::Minute,
            req_per_time_unit: 52,
        };
        assert_eq!(
            Quota::per_minute(NonZeroU32::new(52).unwrap()),
            calculate_quota(rate_limiter_config)
        );
    }

    #[test]
    fn test_calculate_quota_second() {
        let rate_limiter_config = RateLimiterConfig {
            time_unit: TimeUnit::Second,
            req_per_time_unit: 24,
        };
        assert_eq!(
            Quota::per_second(NonZeroU32::new(24).unwrap()),
            calculate_quota(rate_limiter_config)
        );
    }
}
