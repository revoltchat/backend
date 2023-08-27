#[macro_use]
extern crate serde;

#[macro_use]
extern crate async_recursion;

#[macro_use]
extern crate async_trait;

#[macro_use]
extern crate log;

#[macro_use]
extern crate revolt_optional_struct;

#[macro_use]
extern crate revolt_result;

#[cfg(feature = "mongodb")]
pub use mongodb;

#[cfg(feature = "mongodb")]
#[macro_use]
extern crate bson;

macro_rules! database_derived {
    ( $( $item:item )+ ) => {
        $(
            #[derive(Clone)]
            $item
        )+
    };
}

macro_rules! auto_derived {
    ( $( $item:item )+ ) => {
        $(
            #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
            $item
        )+
    };
}

macro_rules! auto_derived_partial {
    ( $item:item, $name:expr ) => {
        #[derive(OptionalStruct, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
        #[optional_derive(Serialize, Deserialize, Debug, Clone, Default, Eq, PartialEq)]
        #[optional_name = $name]
        #[opt_skip_serializing_none]
        #[opt_some_priority]
        $item
    };
}

mod drivers;
pub use drivers::*;

#[cfg(test)]
macro_rules! database_test {
    ( | $db: ident | $test:expr ) => {
        let db = $crate::DatabaseInfo::Test(format!(
            "{}:{}",
            file!().replace('/', "_").replace(".rs", ""),
            line!()
        ))
        .connect()
        .await
        .expect("Database connection failed.");

        db.drop_database().await;

        #[allow(clippy::redundant_closure_call)]
        (|$db: $crate::Database| $test)(db.clone()).await;

        db.drop_database().await
    };
}

mod models;
pub mod util;
pub use models::*;

pub mod events;
pub mod tasks;

/// Utility function to check if a boolean value is false
pub fn if_false(t: &bool) -> bool {
    !t
}
