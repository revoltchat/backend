use revolt_okapi::openapi3::{SecurityScheme, SecuritySchemeData};
use revolt_rocket_okapi::{
    gen::OpenApiGenerator,
    request::{OpenApiFromRequest, RequestHeaderInput},
};

use crate::{AdminAuthorization, AdminMachineToken};

impl OpenApiFromRequest<'_> for AdminAuthorization {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> revolt_rocket_okapi::Result<RequestHeaderInput> {
        let mut requirements = schemars::Map::new();
        requirements.insert("Admin Token".to_owned(), vec![]);

        Ok(RequestHeaderInput::Security(
            "Admin Token".to_owned(),
            SecurityScheme {
                data: SecuritySchemeData::ApiKey {
                    name: "x-admin-user".to_owned(),
                    location: "header".to_owned(),
                },
                description: Some("Used to authenticate as an admin user. 
                Can instead use an x-admin-machine token with an x-on-behalf-of containing the userid or email of 
                the user the machine is performing the action for.".to_owned()),
                extensions: schemars::Map::new(),
            },
            requirements,
        ))
    }
}

impl OpenApiFromRequest<'_> for AdminMachineToken {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> revolt_rocket_okapi::Result<RequestHeaderInput> {
        let mut requirements = schemars::Map::new();
        requirements.insert("Admin Machine Token".to_owned(), vec![]);

        Ok(RequestHeaderInput::Security(
            "Admin Machine Token".to_owned(),
            SecurityScheme {
                data: SecuritySchemeData::ApiKey {
                    name: "x-admin-machine".to_owned(),
                    location: "header".to_owned(),
                },
                description: Some("Used by machines to authenticate on behalf of a user.
                Machines are trusted devices that authenticate as other users via providing the x-on-behalf-of header
                with an admin user's ID or email.".to_owned()),
                extensions: schemars::Map::new(),
            },
            requirements,
        ))
    }
}
