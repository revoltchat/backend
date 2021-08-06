use rocket_governor::{Method, Quota, RocketGovernable, RocketGovernor, NonZeroU32};
use phf::phf_map;

pub struct RateLimitGuard;

static ROUTE_QUOTAS: phf::Map<&'static str, &'static Quota> = phf_map! {
    "message_send" => &Quota::per_second(NonZeroU32::new(1u32).unwrap())
        .allow_burst(NonZeroU32::new(10u32).unwrap())
};

impl<'r> RocketGovernable<'r> for RateLimitGuard {
    fn quota(_method: Method, route: &str) -> Quota {
        if let Some(q) = ROUTE_QUOTAS.get(route) {
            **q
        } else {
            Quota::per_second(Self::nonzero(1u32))
                .allow_burst(Self::nonzero(5u32))
        }
    }
}

pub type RateLimited<'a> = RocketGovernor<'a, RateLimitGuard>;
