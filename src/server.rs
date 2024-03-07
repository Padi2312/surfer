use async_std::fs;
use async_std::net::TcpListener;
use async_std::net::TcpStream;
use async_std::path::PathBuf;
use async_std::prelude::*;
use futures::StreamExt;
use std::collections::HashMap;
use std::pin::Pin;

use crate::logs::Logger;
use crate::request::Method;
use crate::request::Request;
use crate::response::Response;
use crate::utils::get_content_type;

pub struct Server {
    pub address: String,
    pub port: String,
    pub routes: HashMap<String, AsyncHandler>,
    pub static_dirs: HashMap<String, String>,
    logger: Logger,
}

type AsyncHandler = Box<
    dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> + Send + 'static,
>;
// Route macro for registering routes in server
#[macro_export]
macro_rules! route {
    ($method:ident, $path:expr, $handler:expr) => {
        (
            $method,
            $path,
            Box::new(move |request: Request| Box::pin(async move { $handler(request).await })),
        )
    };
}

impl Server {
    pub fn new(address: String, port: String) -> Server {
        Server {
            address,
            port,
            logger: Logger::new(),
            routes: HashMap::new(),
            static_dirs: HashMap::new(),
        }
    }

    pub async fn register_static_dir(&mut self, url_path: &str, dir_path: Option<&str>) {
        let dir_path = PathBuf::from(dir_path.unwrap_or(url_path));
        // Since we're in an async block, we need to lock asynchronously to modify shared state
        self.static_dirs.insert(
            format!("GET {}", url_path),
            dir_path.to_string_lossy().to_string(),
        );
    }

    pub fn register_route(&mut self, data: (Method, &str, AsyncHandler)) {
        let (method, path, handler) = data;
        let index = format!("{} {}", method.to_string(), path);
        self.routes.insert(index, handler);
    }

    pub async fn listen(&self) {
        let listener = TcpListener::bind(format!("{}:{}", self.address, self.port)).await;
        let listener = listener.expect("[ERROR] Failed binding server to address. Exiting...");

        self.logger.info(&format!(
            "Server running at http://{}:{}",
            self.address, self.port
        ));
        self.static_dirs.iter().for_each(|(route, dir)| {
            self.logger.info(&format!(
                "Hosting files from '{}' at {}",
                dir, route
            ));
        });
        self.routes.iter().for_each(|(route, _)| {
            self.logger.info(&format!("Registered route: {}", route));
        });

        listener
            .incoming()
            .for_each_concurrent(None, move |stream| async move {
                match stream {
                    Err(_) => {
                        self.logger.error("Error establishing connection");
                    }
                    Ok(stream) => {
                        self.handle_connection(stream).await;
                    }
                };
            })
            .await;
    }

    pub async fn handle_connection(&self, mut stream: TcpStream) {
        let logger = Logger::new();
        let mut request = Request::new();
        if request.parse(&mut stream).await.is_err() {
            logger.error("Error parsing request");
            self.send_response(
                &mut stream,
                "HTTP/1.1 400 Bad request",
                b"404 Not Found".to_vec(),
                "text/plain",
            )
            .await;
            return;
        }

        logger.info(&format!(
            "{} {} | User-Agent: {}",
            request.method,
            request.path,
            request
                .headers
                .get("User-Agent")
                .unwrap_or(&String::from("N/A"))
        ));

        let route_index = format!("{} {}", request.method.as_str(), request.path.as_str());

        if let Some((request_url_path, dir_path)) = self
            .static_dirs
            .iter()
            .find(|(url_path, _)| route_index.starts_with(url_path.as_str()))
        {
            let relative_path = route_index.trim_start_matches(request_url_path);
            let relative_path = relative_path.split('?').next().unwrap_or(relative_path);
            self.provide_static_dir(PathBuf::from(dir_path), relative_path.to_string())
                .await
                .send(&mut stream)
                .await;
            return;
        }

        // If not found in static_dirs, try to match in routes
        if let Some(route) = self.routes.get(&route_index) {
            route(request).await.send(&mut stream).await;
            return;
        }

        // If no route is found, return 404 Not Found
        self.send_response(
            &mut stream,
            "HTTP/1.1 404 NOT FOUND",
            b"404 Not Found".to_vec(),
            "text/plain",
        )
        .await;
    }

    async fn provide_static_dir(
        &self,
        // request: Request,
        dir_path: PathBuf,
        relative_path: String,
    ) -> Response {
        print!("{:?}", relative_path);
        let mut file_path = dir_path.clone();
        if relative_path.ends_with("/") || relative_path.is_empty() {
            file_path.push("index.html");
        } else {
            file_path.push(relative_path.trim_start_matches('/'));
        }
        let content_type = get_content_type(&file_path);
        let content = if file_path.exists().await && file_path.is_file().await {
            fs::read(&file_path).await.unwrap_or_else(|_| Vec::new())
        } else {
            b"404 Not Found".to_vec()
        };

        let mut headers: HashMap<String, String> = HashMap::new();
        headers.insert("Content-Type".to_string(), content_type.to_string());
        headers.insert("Server".to_string(), "Statiker".to_string());
        Response::new(200, headers, content)
    }

    async fn send_response(
        &self,
        stream: &mut TcpStream,
        status_line: &str,
        content: Vec<u8>,
        content_type: &str,
    ) {
        let response = format!(
            "{}\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n",
            status_line,
            &content.len(),
            content_type
        );
        let response = [response.as_bytes(), &content].concat();
        self.write_response(stream, response).await;
    }

    async fn write_response(&self, stream: &mut TcpStream, response: Vec<u8>) {
        if stream.write_all(&response).await.is_err() {
            self.logger.error("Error writing response");
            return;
        }

        if stream.flush().await.is_err() {
            self.logger.error("Error flushing stream");
            return;
        }
    }
}
