use crate::{
    response::{IntoResponse, Response},
    utils::get_content_type,
};
use async_std::{fs, path::PathBuf};
use std::collections::HashMap;

pub struct FileResponse {
    pub status_code: u16,
    pub file_path: String,
    pub headers: Option<HashMap<String, String>>,
}

impl IntoResponse for FileResponse {
    async fn into_response(self) -> Response {
        let file_path = PathBuf::from(self.file_path);
        let content_type = get_content_type(&file_path);
        let content = if file_path.exists().await && file_path.is_file().await {
            fs::read(&file_path).await.unwrap_or_else(|_| Vec::new())
        } else {
            b"404 Not Found".to_vec()
        };

        let mut headers: HashMap<String, String> = self.headers.unwrap_or(HashMap::new());
        headers.insert("Content-Type".to_string(), content_type.to_string());
        headers.insert("Content-Length".to_string(), content.len().to_string());
        Response {
            status_code: 200,
            headers,
            body: Some(content),
        }
    }
}