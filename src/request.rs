use async_std::net::TcpStream;
use async_std::prelude::*;
use std::collections::HashMap;

pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
    CONNECT,
    TRACE,
}

impl Method {
    pub fn from_string(method: &str) -> Method {
        match method {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "PATCH" => Method::PATCH,
            "HEAD" => Method::HEAD,
            "OPTIONS" => Method::OPTIONS,
            "CONNECT" => Method::CONNECT,
            "TRACE" => Method::TRACE,
            _ => Method::GET,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Method::GET => "GET".to_string(),
            Method::POST => "POST".to_string(),
            Method::PUT => "PUT".to_string(),
            Method::DELETE => "DELETE".to_string(),
            Method::PATCH => "PATCH".to_string(),
            Method::HEAD => "HEAD".to_string(),
            Method::OPTIONS => "OPTIONS".to_string(),
            Method::CONNECT => "CONNECT".to_string(),
            Method::TRACE => "TRACE".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub files: HashMap<String, Vec<u8>>,
    pub http_version: String,
}

impl Request {
    pub fn new() -> Request {
        Request {
            method: String::new(),
            path: String::new(),
            headers: HashMap::new(),
            body: Vec::new(),
            files: HashMap::new(),
            http_version: String::new(),
        }
    }

    pub async fn parse(&mut self, stream: &mut TcpStream) -> Result<(), ()> {
        // First parse the header line to get the method, path and HTTP version
        let mut buffer = [0; 1024];
        let bytes_read = stream.read(&mut buffer).await.unwrap();

        let buffer_request = String::from_utf8_lossy(&buffer[..bytes_read]);
        let mut lines = buffer_request.lines();

        // Parse request line
        let request_line = lines.next();
        if request_line.is_none() {
            return Err(());
        }
        self.process_request_line(&request_line.unwrap());
        self.process_headers(&buffer_request);

        let default_content_type = "text/plain".to_string();
        let content_type = self
            .headers
            .get("Content-Type")
            .unwrap_or(&default_content_type);

        // get content length
        let content_length = self.headers.get("Content-Length");
        let content_length = content_length
            .map(|s| s.parse::<usize>().unwrap())
            .unwrap_or(0);
        let header_length = buffer_request.find("\r\n\r\n").unwrap_or(0) + 4;
        let body_length = bytes_read - header_length;
        if content_length <= body_length {
            let body_buffer = &buffer[header_length..header_length + content_length];
            self.body = body_buffer.to_vec();
        } else {
            let mut body_buffer = vec![0; content_length];
            // Read the beginning of body being already in the buffer
            body_buffer[..body_length].copy_from_slice(&buffer[header_length..bytes_read]);
            // Read the rest of the body from the stream
            stream
                .read_exact(&mut body_buffer[body_length..])
                .await
                .unwrap();
            self.body = body_buffer.to_vec();
        }

        match content_type.split(";").collect::<Vec<&str>>()[0] {
            "application/x-www-form-urlencoded" => {
                let body_buffer = &buffer[header_length..bytes_read];
                self.body = body_buffer.to_vec();
            }
            "multipart/form-data" => {
                let boundary = self
                    .headers
                    .get("Content-Type")
                    .unwrap()
                    .split("boundary=")
                    .collect::<Vec<&str>>()[1];
                self.files = self.parse_multipart_form_data(&self.body, &boundary);
            }
            _ => {
                let body_buffer = &buffer[header_length..bytes_read];
                self.body = body_buffer.to_vec();
            }
        }

        Ok(())
    }

    fn process_request_line(&mut self, request_line: &str) {
        let mut parts = request_line.split_whitespace();
        self.method = parts.next().unwrap_or("N/A").to_string();
        self.path = parts.next().unwrap_or("/").to_string();
        self.http_version = parts.next().unwrap_or("N/A").to_string();
    }

    fn process_headers(&mut self, headers: &str) {
        for line in headers.lines() {
            if line.is_empty() {
                break;
            }

            if let Some((key, value)) = line.split_once(": ") {
                self.headers.insert(key.to_string(), value.to_string());
            }
        }
    }

    fn parse_multipart_form_data(&self, body: &[u8], boundary: &str) -> HashMap<String, Vec<u8>> {
        let mut files: HashMap<String, Vec<u8>> = HashMap::new();
        let boundary = format!("--{}", boundary);
        let body_str = String::from_utf8_lossy(body);

        for part in body_str.split(&boundary) {
            if part.is_empty() || part == "--" {
                continue;
            }

            if let Some((headers_str, content)) = part.split_once("\r\n\r\n") {
                let mut part_headers = HashMap::new();
                let mut name = String::new();
                let mut filename: Option<String> = None;

                for header_line in headers_str.split("\r\n") {
                    if let Some((header_name, header_value)) = header_line.split_once(": ") {
                        match header_name {
                            "Content-Disposition" => {
                                part_headers
                                    .insert(header_name.to_string(), header_value.to_string());

                                // Extract 'name' and potentially 'filename' from Content-Disposition
                                let disposition_parts = header_value
                                    .split(';')
                                    .map(|s| s.trim())
                                    .collect::<Vec<_>>();
                                for part in disposition_parts {
                                    if part.starts_with("name=") {
                                        name = part
                                            .trim_start_matches("name=")
                                            .trim_matches('"')
                                            .to_string();
                                    } else if part.starts_with("filename=") {
                                        filename = Some(
                                            part.trim_start_matches("filename=")
                                                .trim_matches('"')
                                                .to_string(),
                                        );
                                    }
                                }
                            }
                            _ => {
                                // Handle additional headers here
                                part_headers
                                    .insert(header_name.to_string(), header_value.to_string());
                            }
                        }
                    }
                }
                // Process content based on headers (for example, based on Content-Type)
                let trimmed_content = content.trim();
                let content_bytes = trimmed_content.as_bytes().to_vec();
                // Inserting content into the files HashMap. If filename is present, use it as key, else use name.
                let key = filename.unwrap_or_else(|| name.clone());
                if !key.is_empty() {
                    files.insert(key, content_bytes);
                }
            }
        }

        files
    }
}
