use async_std::path::PathBuf;

pub fn get_content_type(file_path: &PathBuf) -> &'static str {
    let ext = file_path
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default();

    match ext {
        "html" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "mp3"|"wav"|"ogg" => "audio/mpeg", 
        _ => "application/octet-stream", // Default binary type
    }
}

