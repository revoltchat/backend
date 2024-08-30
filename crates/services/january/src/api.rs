use axum::{body::Bytes, extract::Query, routing::get, Json, Router};
use revolt_models::v0::Embed;
use revolt_result::Result;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use crate::requests::Request;

pub async fn router() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/proxy", get(proxy))
        .route("/embed", get(embed))
}

/// Successful root response
#[derive(Serialize, Debug, ToSchema)]
pub struct RootResponse {
    january: &'static str,
    version: &'static str,
}

/// Capture crate version from Cargo
static CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Root response from service
#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Echo response", body = RootResponse)
    )
)]
async fn root() -> Json<RootResponse> {
    Json(RootResponse {
        january: "Hello, I am a media proxy server!",
        version: CRATE_VERSION,
    })
}

#[derive(Deserialize)]
struct UrlQuery {
    url: String,
}

/// Proxy a given URL and load media
#[utoipa::path(
    get,
    path = "/proxy",
    responses(
        (status = 200, description = "Requested media file", body = Vec<u8>)
    ),
    params(
        ("url" = String, Query, description = "URL to fetch")
    ),
)]
async fn proxy(Query(UrlQuery { url }): Query<UrlQuery>) -> Result<Bytes> {
    Request::proxy_file(&url).await
}

/// Generate embed for a given URL
#[utoipa::path(
    get,
    path = "/embed",
    responses(
        (status = 200, description = "Generated embed information", body = Embed)
    ),
    params(
        ("url" = String, Query, description = "URL to fetch")
    ),
    security(
        ("api_key" = [])
    )
)]
async fn embed(
    Query(UrlQuery { url }): Query<UrlQuery>,
    TypedHeader(Authorization(_bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<Embed>> {
    Request::generate_embed(&url).await.map(Json)
}
