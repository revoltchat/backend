use tempfile::NamedTempFile;

/// Determine the mime type of the given temporary file and filename
pub fn determine_mime_type(f: &mut NamedTempFile, buf: &[u8], file_name: &str) -> &'static str {
    // Force certain extensions into particular mime types
    if file_name.to_lowercase().ends_with(".apk") {
        return "application/vnd.android.package-archive";
    } else if file_name.to_lowercase().ends_with(".exe") {
        return "application/vnd.microsoft.portable-executable";
    }

    // Use magic signatures to determine mime type
    let kind = infer::get_from_path(f.path()).expect("file read successfully");
    let mime_type = if let Some(kind) = kind {
        kind.mime_type()
    } else {
        "application/octet-stream"
    };

    // See if the file is actually just plain Unicode/ASCII text
    if mime_type == "application/octet-stream" && simdutf8::basic::from_utf8(buf).is_ok() {
        if file_name.to_lowercase().ends_with(".svg") {
            return "image/svg+xml";
        } else {
            return "plain/text";
        }
    }

    mime_type
}
