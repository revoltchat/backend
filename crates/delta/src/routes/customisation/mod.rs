use rocket::Route;
use rocket_okapi::okapi::openapi3::OpenApi;

mod emoji_create;
mod emoji_delete;
mod emoji_fetch;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        emoji_create::create_emoji,
        emoji_delete::delete_emoji,
        emoji_fetch::fetch_emoji
    ]
}
