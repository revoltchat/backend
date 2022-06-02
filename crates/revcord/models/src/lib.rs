mod models;

pub use twilight_model;
pub use models::{channel, user};

use async_trait::async_trait;



#[async_trait]
pub trait QuarkConversion: Sized {
    type Type;

    async fn to_quark(self) -> Self::Type;
    async fn from_quark(data: Self::Type) -> Self;
}

pub fn to_snowflake<T, S: ToString>(ulid: S) -> twilight_model::id::Id<T> {
    todo!()
}

pub fn to_ulid<T>(snowflake: twilight_model::id::Id<T>) -> String {
    todo!()
}
