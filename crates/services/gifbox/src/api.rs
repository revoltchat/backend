use std::sync::Arc;

use axum::{extract::{Query, State}, routing::get, Json, Router};
use revolt_result::{create_error, Result};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use revolt_database::User;

use crate::{AppState, tenor::{Tenor, types}};

pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(root))
        .route("/search", get(search))
}

/// Successful root response
#[derive(Serialize, Debug, ToSchema)]
pub struct RootResponse {
    message: &'static str,
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
        message: "Gifbox lives on!",
        version: CRATE_VERSION,
    })
}

#[derive(Deserialize)]
struct SearchQueryParams {
    pub query: String,
    pub locale: String,
    pub position: Option<String>
}

#[utoipa::path(
    get,
    path = "/search",
    responses(
        (status = 200, description = "Search results", body = SearchResponse)
    )
)]
async fn search(
    _user: User,
    Query(params): Query<SearchQueryParams>,
    State(tenor): State<Tenor>,
) -> Result<Json<Arc<types::SearchResponse>>> {
    // Todo
    tenor.search(&params.query, &params.locale, params.position.as_deref())
        .await
        .map_err(|_| create_error!(InternalError))
        .map(Json)
}