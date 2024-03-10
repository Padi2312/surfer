use crate::response::{IntoResponse, Response};
use std::collections::HashMap;

pub struct HtmlResponse {
    pub status_code: u16,
    pub content: String,
    pub headers: Option<HashMap<String, String>>,
}

impl IntoResponse for HtmlResponse {
    async fn into_response(self) -> Response {
        let mut headers: HashMap<String, String> = self.headers.unwrap_or(HashMap::new());
        headers.insert("Content-Type".to_string(), "text/html".to_string());
        Response {
            status_code: self.status_code,
            headers,
            body: Some(self.content.into_bytes()),
        }
    }
}