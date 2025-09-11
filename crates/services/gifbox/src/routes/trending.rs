use axum::{
    extract::{Query, State},
    Json,
};
use revolt_database::User;
use revolt_result::{create_error, Result};
use serde::{Deserialize};
use utoipa::{IntoParams};

use crate::{
    tenor,
    types,
};

#[derive(Deserialize, IntoParams)]
pub struct TrendingQueryParams {
    #[param(example = "en_US")]
    pub locale: String,
    pub limit: Option<u32>,
    pub position: Option<String>
}

/// Trending GIFs
#[utoipa::path(
    get,
    path = "/featured",
    tag = "GIFs",
    security(("User Token" = []), ("Bot Token" = [])),
    params(TrendingQueryParams),
    responses(
        (status = 200, description = "Trending results", body = inline(types::PaginatedMediaResponse))
    )
)]
pub async fn trending(
    _user: User,
    Query(params): Query<TrendingQueryParams>,
    State(tenor): State<tenor::Tenor>,
) -> Result<Json<types::PaginatedMediaResponse>> {
    tenor
        .featured(&params.locale, params.limit.unwrap_or(50), params.position.as_deref().unwrap_or_default())
        .await
        .map_err(|_| create_error!(InternalError))
        .map(|results| results.as_ref().clone().into())
        .map(Json)
}
