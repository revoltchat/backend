use std::{io::Cursor, time::Duration};

use axum::{
    extract::{DefaultBodyLimit, Path, State},
    http::header,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Json, Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use image::ImageReader;
use lazy_static::lazy_static;
use revolt_config::config;
use revolt_database::{Database, FileHash, Metadata};
use revolt_files::fetch_from_s3;
use revolt_result::{create_error, Result};
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use utoipa::ToSchema;

use crate::{
    metadata::{generate_metadata, temp_file_size},
    mime_type::determine_mime_type,
};

/// Build the API router
pub async fn router() -> Router<Database> {
    let config = config().await;

    Router::new()
        .route("/", get(root))
        .route(
            "/:tag",
            post(upload_file).layer(DefaultBodyLimit::max(
                config.features.limits.global.body_limit_size,
            )),
        )
        .route("/:tag/:file_id", get(fetch_preview))
        .route("/:tag/:file_id/:file_name", get(fetch_file))
}

lazy_static! {
    /// Short-lived file cache to allow us to populate different CDN regions without increasing bandwidth to S3 provider
    /// Uploads will also be stored here to prevent immediately queued downloads from doing the entire round-trip
    static ref S3_CACHE: moka::future::Cache<String, Result<Vec<u8>>> = moka::future::Cache::builder()
        .max_capacity(10_000) // TODO config
        .time_to_live(Duration::from_secs(60)) // TODO config
        .build();
}

/// Retrieve hash information and file data by given hash
async fn retrieve_file_by_hash(hash: &FileHash) -> Result<Vec<u8>> {
    if let Some(data) = S3_CACHE.get(&hash.id).await {
        data
    } else {
        let data = fetch_from_s3(&hash.bucket_id, &hash.path, &hash.iv).await;
        S3_CACHE.insert(hash.id.to_owned(), data.clone()).await;
        data
    }
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
#[derive(Deserialize, Debug, ToSchema, strum_macros::IntoStaticStr)]
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
    TypedMultipart(UploadPayload { mut file }): TypedMultipart<UploadPayload>,
) -> axum::response::Result<Json<UploadResponse>> {
    // Extract the filename, or give it a generic name.
    let file_name = file.metadata.file_name.unwrap_or("unnamed-file".to_owned());

    // TODO: find existing `hash` (or match `processed`) and use that if possible
    // then: create attachment and return UploadResponse { id }

    // Determine file size
    let original_file_size = temp_file_size(&file.contents)?;

    // Determine the mime type for the file
    let mime_type = determine_mime_type(&mut file.contents, &file_name, original_file_size);

    // TODO: block mime types here

    // Determine metadata for the file
    let metadata = generate_metadata(&file.contents, mime_type);

    // TODO: Re-encode MIMES_WITH_EXIF to remove EXIF data (this needs orientation data: https://github.com/revoltchat/autumn/blob/master/src/routes/upload.rs#L96)
    // TODO: Re-encode videos to remove EXIF data (this may need orientation data?)

    // TODO: Virus scan Metadata::Text/File

    // TODO: encrypt the file and generate FileHash
    // note: file_size is processed file's size (incl. authTag in encrypted buffer)
    // TODO: insert FileHash
    // TODO: upload to S3
    // TODO: create attachment

    Ok(Json(UploadResponse { id: "aaa" }))
}

/// Header value used for cache control
pub static CACHE_CONTROL: &str = "public, max-age=604800, must-revalidate";

/// Fetch preview of file
///
/// This route will only return image content.
/// For all other file types, please use the fetch route (you will receive a redirect if you try to use this route anyways!).
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
    State(db): State<Database>,
    Path((tag, file_id)): Path<(Tag, String)>,
) -> Result<Response> {
    let tag: &'static str = tag.into();
    let file = db.fetch_attachment(tag, &file_id).await?;

    // Ignore deleted files
    if file.deleted.is_some_and(|v| v) {
        return Err(create_error!(NotFound));
    }

    // Ignore files that haven't been attached
    if file.used_for.is_none() {
        return Err(create_error!(NotFound));
    }

    let hash = file.as_hash(&db).await?;

    // Only process image files
    if !matches!(hash.metadata, Metadata::Image { .. }) {
        return Ok(
            Redirect::permanent(&format!("/{tag}/{file_id}/{}", file.filename)).into_response(),
        );
    }

    // Original image data
    let data = retrieve_file_by_hash(&hash).await?;

    // Dimensions we need to resize to
    let config = config().await;
    let [w, h] = config.files.preview.get(tag).unwrap();

    // Read the image and resize it
    // TODO: use jxl_oxide to process image/jxl files
    let image = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|_| create_error!(InternalError))?
        .decode()
        .map_err(|_| create_error!(InternalError))?
        //.resize(width as u32, height as u32, image::imageops::FilterType::Gaussian)
        // resize is about 2.5x slower,
        // thumbnail doesn't have terrible quality
        // so we use thumbnail
        .thumbnail(*w as u32, *h as u32);

    // Encode it into WEBP
    let encoder = webp::Encoder::from_image(&image).expect("Could not create encoder.");
    let data = if config.files.webp_quality != 100.0 {
        encoder.encode(config.files.webp_quality).to_vec()
    } else {
        encoder.encode_lossless().to_vec()
    };

    Ok((
        [
            (header::CONTENT_TYPE, "image/webp"),
            (header::CONTENT_DISPOSITION, "inline"),
            (header::CACHE_CONTROL, CACHE_CONTROL),
        ],
        data,
    )
        .into_response())
}

/// Fetch original file
///
/// Content disposition header will be set to 'attachment' to prevent browser from rendering anything.
#[utoipa::path(
    get,
    path = "/{tag}/{file_id}/{file_name}",
    responses(
        (status = 200, description = "Original file", body = Vec<u8>)
    ),
    params(
        ("tag" = Tag, Path, description = "Tag to fetch from (e.g. attachments, icons, ...)"),
        ("file_id" = String, Path, description = "File identifier"),
        ("file_name" = String, Path, description = "File name")
    ),
)]
async fn fetch_file(
    State(db): State<Database>,
    Path((tag, file_id, file_name)): Path<(Tag, String, String)>,
) -> Result<Response> {
    let file = db.fetch_attachment(tag.into(), &file_id).await?;

    // Ignore deleted files
    if file.deleted.is_some_and(|v| v) {
        return Err(create_error!(NotFound));
    }

    // Ignore files that haven't been attached
    if file.used_for.is_none() {
        return Err(create_error!(NotFound));
    }

    // Ensure filename is correct
    if file_name != file.filename {
        return Err(create_error!(NotFound));
    }

    let hash = file.as_hash(&db).await?;
    retrieve_file_by_hash(&hash).await.map(|data| {
        (
            [
                (header::CONTENT_TYPE, hash.content_type),
                (header::CONTENT_DISPOSITION, "attachment".to_owned()),
                (header::CACHE_CONTROL, CACHE_CONTROL.to_owned()),
            ],
            data,
        )
            .into_response()
    })
}
