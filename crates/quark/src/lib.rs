#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate schemars;
#[macro_use]
extern crate async_recursion;
#[macro_use]
extern crate log;
#[macro_use]
extern crate impl_ops;
#[macro_use]
extern crate optional_struct;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitfield;
#[macro_use]
pub extern crate bson;

pub use iso8601_timestamp::Timestamp;
pub use rauth;
pub use redis_kiss;

pub mod events;
pub mod r#impl;
pub mod models;
pub mod presence;
pub mod tasks;
pub mod types;

#[cfg(feature = "rocket_impl")]
pub mod web;

mod database;
mod permissions;
mod traits;
mod util;

pub use database::*;
pub use traits::*;

pub use permissions::defn::*;
pub use permissions::{get_relationship, perms};

pub use util::{
    log::setup_logging,
    r#ref::Ref,
    result::{Error, Result},
    variables,
};

pub use web::{Db, EmptyResponse};
