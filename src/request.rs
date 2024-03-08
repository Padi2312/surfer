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
pub struct FormData {
    pub name: String,
    pub filename: Option<String>,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub form_data: Vec<FormData>,
    pub http_version: String,
}

impl Request {
    pub fn new() -> Request {
        Request {
            method: String::new(),
            path: String::new(),
            headers: HashMap::new(),
            body: Vec::new(),
            form_data: Vec::new(),
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
                self.form_data = self.parse_multipart_form_data(&self.body, &boundary);
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
    fn parse_multipart_form_data(&self, body: &[u8], boundary: &str) -> Vec<FormData> {
        let mut form_data: Vec<FormData> = Vec::new();
        let boundary = format!("--{}", boundary).into_bytes();
        let mut start_index = 0;

        // We need to iterate manually over the bytes until we find the boundary
        while let Some(boundary_index) = self.find_boundary(&body[start_index..], &boundary) {
            let part_start = start_index + boundary_index + boundary.len();
            if let Some(next_boundary_index) = self.find_boundary(&body[part_start..], &boundary) {
                let part = &body[part_start..part_start + next_boundary_index];
                if let Some(data) = self.process_part(part) {
                    form_data.push(data);
                }
                start_index = part_start + next_boundary_index;
            } else {
                break;
            }
        }
        form_data
    }

    fn find_boundary(&self, data: &[u8], boundary: &[u8]) -> Option<usize> {
        data.windows(boundary.len())
            .position(|window| window == boundary)
    }

    fn process_part(&self, part: &[u8]) -> Option<FormData> {
        let split_index = part.windows(4).position(|window| window == b"\r\n\r\n")?;
        // Split the part into headers and content (because content should be handled differently depending on the headers)
        let (header_part, content_part) = part.split_at(split_index);
        let headers_str = String::from_utf8_lossy(&header_part);
        let (name, filename) = self.parse_multipart_form_data_headers(&headers_str);

        Some(FormData {
            name,
            filename,
            data: content_part[4..].to_vec(), // Skip the "\r\n\r\n"
        })
    }

    fn parse_multipart_form_data_headers(&self, headers: &str) -> (String, Option<String>) {
        let mut name = String::new();
        let mut filename = None;
        for line in headers.lines() {
            if line.starts_with("Content-Disposition:") {
                for attr in line.split(';').skip(1) {
                    let attr = attr.trim();
                    if attr.starts_with("name=") {
                        name = attr
                            .trim_start_matches("name=\"")
                            .trim_end_matches("\"")
                            .to_string();
                    } else if attr.starts_with("filename=") {
                        filename = Some(
                            attr.trim_start_matches("filename=\"")
                                .trim_end_matches("\"")
                                .to_string(),
                        );
                    }
                }
            }
        }
        (name, filename)
    }
}
