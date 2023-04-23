#[macro_use]
extern crate auto_ops;

#[macro_use]
extern crate async_trait;

mod r#impl;
mod models;
mod r#trait;

pub use models::*;
pub use r#impl::*;
pub use r#trait::*;

#[cfg(test)]
mod test;
