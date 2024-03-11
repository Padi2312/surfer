use crate::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonResponse<T> {
    pub status_code: u16,
    pub body: T,
    pub headers: Option<HashMap<String, String>>,
}

impl<T: serde::Serialize + Send> IntoResponse for JsonResponse<T> {
    // Add trait bounds for Serialize and Send
    async fn into_response(self) -> Response
    where
        T: Send,
    {
        let body = serde_json::to_vec(&self.body).unwrap();
        let mut headers = self.headers.unwrap_or(HashMap::new());
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Content-Length".to_string(), body.len().to_string());
        Response::new(self.status_code)
            .with_body(body)
            .with_headers(headers)
    }
}
