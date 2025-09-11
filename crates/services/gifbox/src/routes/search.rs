use axum::{
    extract::{Query, State},
    Json,
};
use revolt_database::User;
use revolt_result::{create_error, Result};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{tenor, types};

#[derive(Deserialize, IntoParams)]
pub struct SearchQueryParams {
    #[param(example = "Wave")]
    pub query: String,
    #[param(example = "en_US")]
    pub locale: String,
    pub limit: Option<u32>,
    pub is_category: Option<bool>,
    pub position: Option<String>,
}

/// Searches for GIFs with a query
#[utoipa::path(
    get,
    path = "/search",
    tag = "GIFs",
    security(("User Token" = []), ("Bot Token" = [])),
    params(SearchQueryParams),
    responses(
        (status = 200, description = "Search results", body = inline(types::PaginatedMediaResponse))
    )
)]
pub async fn search(
    _user: User,
    Query(params): Query<SearchQueryParams>,
    State(tenor): State<tenor::Tenor>,
) -> Result<Json<types::PaginatedMediaResponse>> {
    tenor
        .search(
            &params.query,
            &params.locale,
            params.limit.unwrap_or(50),
            params.is_category.unwrap_or_default(),
            params.position.as_deref().unwrap_or_default(),
        )
        .await
        .map_err(|_| create_error!(InternalError))
        .map(|results| results.as_ref().clone().into())
        .map(Json)
}
