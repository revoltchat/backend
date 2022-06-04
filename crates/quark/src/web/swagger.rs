use rocket::Route;

pub fn routes() -> Vec<Route> {
    rocket_okapi::swagger_ui::make_swagger_ui(&rocket_okapi::swagger_ui::SwaggerUIConfig {
        url: "../openapi.json".to_owned(),
        ..Default::default()
    })
    .into()
}
