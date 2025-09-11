use crate::AppState;
use axum::routing::{get, Router};

pub mod categories;
pub mod root;
pub mod search;
pub mod trending;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(root::root))
        .route("/categories", get(categories::categories))
        .route("/search", get(search::search))
        .route("/trending", get(trending::trending))
}
