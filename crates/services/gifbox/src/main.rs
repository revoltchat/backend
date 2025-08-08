use std::net::{Ipv4Addr, SocketAddr};

use axum::{extract::FromRef, Router};

use revolt_config::config;
use revolt_database::{Database, DatabaseInfo};
use tokio::net::TcpListener;
use utoipa::{
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_scalar::{Scalar, Servable as ScalarServable};

use crate::tenor::Tenor;

mod api;
mod tenor;
mod types;

#[derive(Clone)]
struct AppState {
    pub database: Database,
    pub tenor: Tenor,
}

impl FromRef<AppState> for Database {
    fn from_ref(state: &AppState) -> Self {
        state.database.clone()
    }
}

impl FromRef<AppState> for Tenor {
    fn from_ref(state: &AppState) -> Self {
        state.tenor.clone()
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Configure logging and environment
    revolt_config::configure!(gifbox);

    // Configure API schema
    #[derive(OpenApi)]
    #[openapi(
        modifiers(&SecurityAddon),
        paths(
            api::root,
            api::search,
        ),
        components(
            schemas(
                revolt_result::Error,
                revolt_result::ErrorType,
                types::SearchResponse,
                types::MediaObject,
                types::MediaResult,
            )
        )
    )]
    struct ApiDoc;

    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            if let Some(components) = openapi.components.as_mut() {
                components.add_security_scheme(
                    "api_key",
                    SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
                )
            }
        }
    }

    let config = config().await;
    let state = AppState {
        database: DatabaseInfo::Auto
            .connect()
            .await
            .expect("Unable to connect to database"),
        tenor: tenor::Tenor::new(&config.api.security.tenor_key),
    };

    // Configure Axum and router
    let app = Router::new()
        .merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
        .nest("/", api::router().await)
        .with_state(state);

    // Configure TCP listener and bind
    tracing::info!("Listening on 0.0.0.0:14706");
    tracing::info!("Play around with the API: http://localhost:14706/scalar");
    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 14706));
    let listener = TcpListener::bind(&address).await?;
    axum::serve(listener, app.into_make_service()).await
}
