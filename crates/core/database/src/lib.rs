#[macro_use]
extern crate serde;

#[macro_use]
extern crate async_recursion;

#[macro_use]
extern crate async_trait;

#[macro_use]
extern crate log;

#[macro_use]
extern crate optional_struct;

#[cfg(feature = "mongodb")]
pub use mongodb;

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
            #[derive(Serialize, Deserialize, Debug, Clone)]
            $item
        )+
    };
}

macro_rules! auto_derived_partial {
    ( $item:item, $name:expr ) => {
        #[derive(OptionalStruct, Serialize, Deserialize, Debug, Clone)]
        #[optional_derive(Serialize, Deserialize, Debug, Clone)]
        #[optional_name = $name]
        #[opt_skip_serializing_none]
        #[opt_some_priority]
        $item
    };
}

mod drivers;
pub use drivers::*;

mod models;
pub use models::*;

/// Utility function to check if a boolean value is false
pub fn if_false(t: &bool) -> bool {
    !t
}
