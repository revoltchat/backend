use std::marker::PhantomData;

pub struct OAuth2Scoped<Scope> {
    pub(crate) _scope: PhantomData<Scope>
}

pub trait OAuth2Scope {
    const NAME: &'static str;
    const SCOPE: crate::OAuth2Scope;
}

macro_rules! define_oauth2_scope {
    ($struct_name:ident, $name:literal) => {
        pub struct $struct_name;

        impl OAuth2Scope for $struct_name {
            const NAME: &'static str = $name;
            const SCOPE: crate::OAuth2Scope = crate::OAuth2Scope::$struct_name;
        }
    };
}

define_oauth2_scope!(ReadIdentify, "read:identify");
define_oauth2_scope!(ReadServers, "read:servers");
define_oauth2_scope!(WriteFiles, "write:files");
define_oauth2_scope!(Events, "events");
define_oauth2_scope!(Full, "full");
