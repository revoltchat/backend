pub mod json;
pub mod validator;

pub use json::Json;
pub use validator::Validate;

#[macro_export]
macro_rules! fn_request_body {
    ($gen:ident, $ty:path, $mime_type:expr) => {{
        let schema = $gen.json_schema::<$ty>();
        Ok(revolt_rocket_okapi::revolt_okapi::openapi3::RequestBody {
            content: {
                let mut map = revolt_rocket_okapi::revolt_okapi::Map::new();
                map.insert(
                    $mime_type.to_owned(),
                    revolt_rocket_okapi::revolt_okapi::openapi3::MediaType {
                        schema: Some(schema),
                        ..Default::default()
                    },
                );
                map
            },
            required: true,
            ..Default::default()
        })
    }};
}