use std::io::Read;

use tempfile::NamedTempFile;

/// Determine the mime type of the given temporary file and filename
pub fn determine_mime_type(f: &mut NamedTempFile, file_name: &str, file_size: u64) -> &'static str {
    // Use magic signatures to determine mime type
    let kind = infer::get_from_path(f.path()).expect("file read successfully");
    let mime_type = if let Some(kind) = kind {
        kind.mime_type()
    } else {
        "application/octet-stream"
    };

    // Map any known conflicts where appropriate
    let mime_type = if mime_type == "application/zip" && file_name.to_lowercase().ends_with(".apk")
    {
        "application/vnd.android.package-archive"
    } else {
        mime_type
    };

    // See if the file is actually just plain Unicode/ASCII text
    if mime_type == "application/octet-stream" {
        // don't check files over >= 500 kB
        if file_size <= 500_000 {
            let mut buf = String::new();
            if f.read_to_string(&mut buf).is_ok() {
                // successfully read the file as UTF-8
                return "plain/text";
            }
        }
    }

    mime_type
}
