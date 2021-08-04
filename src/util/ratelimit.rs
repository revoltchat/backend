use std::time::Duration;

use rocket_governor::{Method, Quota, RocketGovernable};

pub struct RateLimitGuard;

impl<'r> RocketGovernable<'r> for RateLimitGuard {
    fn quota(_method: Method, _route_name: &str) -> Quota {
        dbg!(_method, _route_name);
        Quota::per_second(Self::nonzero(1u32))
            .allow_burst(Self::nonzero(10u32))
    }
}
