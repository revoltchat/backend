use rocket::Route;

pub fn routes() -> Vec<Route> {
    revolt_rocket_okapi::swagger_ui::make_swagger_ui(&revolt_rocket_okapi::swagger_ui::SwaggerUIConfig {
        url: "../openapi.json".to_owned(),
        ..Default::default()
    })
    .into()
}
