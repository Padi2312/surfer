pub mod file_response;
pub mod html_response;
pub mod json_response;

use async_std::{net::TcpStream, prelude::*};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Response {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

pub trait IntoResponse {
    fn into_response(self) -> impl std::future::Future<Output = Response> + Send;
}

impl Response {
    pub async fn send(&mut self, stream: &mut TcpStream) {
        let mut response = format!("HTTP/1.1 {}\r\n", self.status_code);
        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }
        response.push_str("\r\n");

        let mut response = response.as_bytes().to_vec();
        if self.body.is_some() {
            response = [
                response,
                <Option<Vec<u8>> as Clone>::clone(&self.body).unwrap(),
            ]
            .concat();
        }

        if stream.write_all(&response).await.is_err() {
            println!("Error writing response");
            return;
        }

        if stream.flush().await.is_err() {
            println!("Error flushing response");
            return;
        }
    }
}
