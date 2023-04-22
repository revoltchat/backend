#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

#[cfg(feature = "schemas")]
#[macro_use]
extern crate schemars;

macro_rules! auto_derived {
    ( $( $item:item )+ ) => {
        $(
            #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
            #[cfg_attr(feature = "schemas", derive(JsonSchema))]
            #[derive(Debug, Clone, Eq, PartialEq)]
            $item
        )+
    };
}

mod bots;

pub use bots::*;

/// Utility function to check if a boolean value is false
pub fn if_false(t: &bool) -> bool {
    !t
}
