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
        Response::new(self.status_code)
            .with_body(self.content.into_bytes())
            .with_headers(headers)
    }
}