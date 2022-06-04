use rocket::http::ContentType;
use rocket::response::{self, Responder};
use rocket::{Request, Response};
use rocket_okapi::okapi::openapi3::{self, MediaType, RefOr};
use schemars::schema::{InstanceType, SchemaObject, SingleOrVec};

pub struct CachedFile((ContentType, Vec<u8>));

pub static CACHE_CONTROL: &str = "public, max-age=31536000, immutable";

impl<'r> Responder<'r, 'static> for CachedFile {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        Response::build_from(self.0.respond_to(req)?)
            .raw_header("Cache-Control", CACHE_CONTROL)
            .ok()
    }
}

impl rocket_okapi::response::OpenApiResponderInner for CachedFile {
    fn responses(
        _gen: &mut rocket_okapi::gen::OpenApiGenerator,
    ) -> std::result::Result<openapi3::Responses, rocket_okapi::OpenApiError> {
        let mut responses = schemars::Map::new();
        let mut content = schemars::Map::new();

        content.insert(
            "image/png".to_owned(),
            MediaType {
                schema: Some(SchemaObject {
                    instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::String))),
                    format: Some("binary".to_owned()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        );

        responses.insert(
            "200".to_string(),
            RefOr::Object(openapi3::Response {
                description: "Default Avatar Picture".to_string(),
                content,
                ..Default::default()
            }),
        );

        Ok(openapi3::Responses {
            responses,
            ..Default::default()
        })
    }
}

/// # Fetch Default Avatar
///
/// This returns a default avatar based on the given id.
#[openapi(tag = "User Information")]
#[get("/<target>/default_avatar")]
pub async fn req(target: String) -> CachedFile {
    CachedFile((
        ContentType::PNG,
        revolt_quark::util::pfp::avatar(target.chars().last().unwrap()),
    ))
}
