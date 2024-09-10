use std::io::Cursor;

use revolt_database::Metadata;
use tempfile::NamedTempFile;

/// Intersection of what infer can detect and what image-rs supports
///
/// Note: imagesize crate also supports all of these, so we use that for quick size probing.
static SUPPORTED_IMAGE_MIME: [&str; 9] = [
    "image/avif",
    "image/bmp",
    "image/gif",
    "image/vnd.microsoft.icon",
    "image/jpeg",
    "image/jxl", // not supported by image-rs but we shim it
    "image/png",
    "image/tiff",
    "image/webp",
];

/// Generate metadata from file, using mime type as a hint
pub fn generate_metadata(f: &NamedTempFile, mime_type: &str) -> Metadata {
    if SUPPORTED_IMAGE_MIME.contains(&mime_type) {
        if let Ok(size) = imagesize::size(f.path())
            .inspect_err(|err| tracing::error!("Failed to generate image size! {err:?}"))
        {
            if let (Ok(width), Ok(height)) = (size.width.try_into(), size.height.try_into()) {
                return Metadata::Image { width, height };
            }
        }

        Metadata::File
    } else if mime_type.starts_with("video/") {
        if let Ok(data) = ffprobe::ffprobe(f.path())
            .inspect_err(|err| tracing::error!("Failed to ffprobe file! {err:?}"))
        {
            // Use first valid stream
            for stream in data.streams {
                if let (Some(w), Some(h)) = (stream.width, stream.height) {
                    if let (Ok(width), Ok(height)) = (w.try_into(), h.try_into()) {
                        return Metadata::Video { width, height };
                    }
                }
            }

            Metadata::File
        } else {
            Metadata::File
        }
    } else if mime_type.starts_with("audio/") {
        Metadata::Audio
    } else if mime_type == "plain/text" {
        Metadata::Text
    } else {
        Metadata::File
    }
}

/// Subroutine to ensure data isn't corrupted
pub fn validate_from_metadata(
    reader: Cursor<Vec<u8>>,
    metadata: Metadata,
    mime_type: &str,
) -> Metadata {
    if let Metadata::Image { .. } = &metadata {
        if mime_type == "image/jxl" {
            // Check if we can read using jxl-oxide crate
            if jxl_oxide::JxlImage::builder()
                .read(reader)
                .inspect_err(|err| tracing::error!("Failed to read JXL! {err:?}"))
                .is_err()
            {
                return Metadata::File;
            }
        } else if matches!(
            // Check if we can read using image-rs crate
            image::ImageReader::new(reader)
                .with_guessed_format()
                .inspect_err(|err| tracing::error!("Failed to read image! {err:?}"))
                .map(|f| f.decode()),
            Err(_) | Ok(Err(_))
        ) {
            return Metadata::File;
        }
    }

    metadata
}
