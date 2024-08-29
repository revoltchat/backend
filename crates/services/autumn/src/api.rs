use axum::{
    extract::{DefaultBodyLimit, Multipart, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use revolt_config::config;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use tempfile::NamedTempFile;

pub async fn router() -> Router {
    let config = config().await;

    Router::new().route("/", get(root)).route(
        "/:tag",
        post(upload_file).layer(DefaultBodyLimit::max(
            config.features.limits.global.body_limit_size,
        )),
    )
}

/// Successful root response
#[derive(Serialize, Debug, ToSchema)]
pub struct RootResponse {
    autumn: &'static str,
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
        autumn: "Hello, I am a file server!",
        version: CRATE_VERSION,
    })
}

/// Available tags to upload to
#[derive(Deserialize, Debug, ToSchema)]
#[allow(non_camel_case_types)]
pub enum Tag {
    attachments,
    avatars,
    backgrounds,
    icons,
    banners,
    emojis,
}

/// Request body for upload
#[derive(ToSchema, TryFromMultipart)]
pub struct UploadPayload {
    #[schema(format = Binary)]
    #[allow(dead_code)]
    #[form_data(limit = "unlimited")] // handled by axum
    file: FieldData<NamedTempFile>,
}

/// Successful upload response
#[derive(Serialize, Debug, ToSchema)]
pub struct UploadResponse {
    /// ID to attach uploaded file to object
    id: &'static str,
}

/// Upload a file
///
/// Available tags and restrictions:
///
/// | Tag | Size | Resolution | Type |
/// | :-: | --: | :-- | :-: |
/// | attachments | 20 MB | - | Any |
/// | avatars | 4 MB | 40 MP or 10,000px | Image |
/// | backgrounds | 6 MB | 40 MP or 10,000px | Image |
/// | icons | 2.5 MB | 40 MP or 10,000px | Image |
/// | banners | 6 MB | 40 MP or 10,000px | Image |
/// | emojis | 500 KB | 40 MP or 10,000px | Image |
#[utoipa::path(
    post,
    path = "/{tag}",
    responses(
        (status = 200, description = "Upload was successful", body = UploadResponse)
    ),
    params(
        ("tag" = Tag, Path, description = "Tag to upload to (e.g. attachments, icons, ...)")
    ),
    request_body(content_type = "multipart/form-data", content = UploadPayload),
)]
async fn upload_file(
    Path(tag): Path<Tag>,
    TypedMultipart(UploadPayload { file }): TypedMultipart<UploadPayload>,
) -> axum::response::Result<Json<UploadResponse>> {
    Ok(Json(UploadResponse { id: "aaa" }))
}

/// Fetch preview of file
///
/// Depending on the given tag, the file will be re-processed to fit the criteria:
///
/// | Tag | Image Resolution <sup>†</sup> |
/// | :-: | --- |
/// | attachments | Up to 1280px on any axis |
/// | avatars | Up to 128px on any axis |
/// | backgrounds | Up to 1280x720px |
/// | icons | Up to 128px on any axis |
/// | banners | Up to 480px on any axis |
/// | emojis | Up to 128px on any axis |
///
/// <sup>†</sup> aspect ratio will always be preserved
#[utoipa::path(
    get,
    path = "/{tag}/{file_id}",
    responses(
        (status = 200, description = "Generated preview", body = Vec<u8>)
    ),
    params(
        ("tag" = Tag, Path, description = "Tag to fetch from (e.g. attachments, icons, ...)"),
        ("file_id" = String, Path, description = "File identifier")
    ),
)]
async fn fetch_preview(
    Path(tag): Path<Tag>,
    Path(file_id): Path<String>,
) -> axum::response::Result<Response> {
    todo!()
}

/// Fetch original file
#[utoipa::path(
    get,
    path = "/{tag}/{file_id}/{file_name}",
    responses(
        (status = 200, description = "Generated preview", body = Vec<u8>)
    ),
    params(
        ("tag" = Tag, Path, description = "Tag to fetch from (e.g. attachments, icons, ...)"),
        ("file_id" = String, Path, description = "File identifier"),
        ("file_name" = String, Path, description = "File name")
    ),
)]
async fn fetch_file(
    Path(tag): Path<Tag>,
    Path(file_id): Path<String>,
    Path(file_name): Path<String>,
) -> axum::response::Result<Response> {
    todo!()
}
