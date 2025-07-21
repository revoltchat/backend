use std::marker::PhantomData;

pub struct OAuth2Scoped<Scope> {
    pub(crate) _scope: PhantomData<Scope>
}

pub trait OAuth2Scope {
    const SCOPE: crate::OAuth2Scope;
    const MODEL: revolt_models::v0::OAuth2Scope;
}

macro_rules! define_oauth2_scope {
    ($struct_name:ident) => {
        pub struct $struct_name;

        impl OAuth2Scope for $struct_name {
            const SCOPE: crate::OAuth2Scope = crate::OAuth2Scope::$struct_name;
            const MODEL: revolt_models::v0::OAuth2Scope = revolt_models::v0::OAuth2Scope::$struct_name;
        }
    };
}

// This must match the OAuth2Scope enum
// TODO: automatically sync this
define_oauth2_scope!(ReadIdentify);
define_oauth2_scope!(ReadServers);
define_oauth2_scope!(WriteFiles);
define_oauth2_scope!(Events);
define_oauth2_scope!(Full);
