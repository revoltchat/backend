#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

#[cfg(feature = "schemas")]
#[macro_use]
extern crate schemars;

#[cfg(feature = "utoipa")]
#[macro_use]
extern crate utoipa;

#[cfg(feature = "partials")]
#[macro_use]
extern crate revolt_optional_struct;

#[cfg(feature = "validator")]
pub use validator;

macro_rules! auto_derived {
    ( $( $item:item )+ ) => {
        $(
            #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
            #[cfg_attr(feature = "schemas", derive(JsonSchema))]
            #[cfg_attr(feature = "utoipa", derive(ToSchema))]
            #[derive(Debug, Clone, Eq, PartialEq)]
            $item
        )+
    };
}

#[cfg(feature = "partials")]
macro_rules! auto_derived_partial {
    ( $item:item, $name:expr ) => {
        #[derive(
            OptionalStruct, Debug, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema,
        )]
        #[optional_derive(
            Debug,
            Clone,
            Eq,
            PartialEq,
            Serialize,
            Deserialize,
            JsonSchema,
            Default
        )]
        #[optional_name = $name]
        #[opt_skip_serializing_none]
        #[opt_some_priority]
        $item
    };
}

#[cfg(not(feature = "partials"))]
macro_rules! auto_derived_partial {
    ( $item:item, $name:expr ) => {
        #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
        $item
    };
}

pub mod v0;

/// Utility function to check if a boolean value is false
pub fn if_false(t: &bool) -> bool {
    !t
}

/// Utility function to check if an u32 is zero
pub fn if_zero_u32(t: &u32) -> bool {
    t == &0
}

/// Utility function to check if an option doesnt contain true
pub fn if_option_false(t: &Option<bool>) -> bool {
    t != &Some(true)
}
