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

use crate::response::file_response::FileResponse;
use crate::response::IntoResponse;
use crate::response::Response;

pub struct Server {
    pub address: String,
    pub port: String,
    pub routes: HashMap<String, AsyncHandler>,
    pub static_dirs: HashMap<String, String>,
    logger: Logger,
}

pub type AsyncHandler = Box<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response>>>>;
// Route macro for registering routes in server
#[macro_export]
macro_rules! route {
    // Pattern for routes with additional arguments
    ($method:ident, $path:expr, $handler:expr, $arg:expr) => {{
        use std::collections::HashMap;
        let cloned_arg = $arg.clone(); // Clone outside the closure
        (
            $method,
            $path,
            Box::new(move |request: Request| {
                let arg_for_async = cloned_arg.clone(); // Clone for the async block
                Box::pin(async move { $handler(request, arg_for_async).await })
            }),
        )
    }};
    // Pattern for routes without additional arguments
    ($method:ident, $path:expr, $handler:expr) => {
        (
            $method,
            $path,
            Box::new(move |request: Request| Box::pin(async move { $handler(request).await })),
        )
    };
}

#[macro_export]
macro_rules! headers {
    ($(($key:expr, $value:expr)),*) => {{
        use std::collections::HashMap;
        let mut headers = HashMap::new();
        $(
            headers.insert($key.to_string(), $value.to_string());
        )*
        headers
    }};
}

impl Server {
    pub fn new(address: Option<String>, port: Option<String>) -> Server {
        Server {
            address: address.unwrap_or("0.0.0.0".to_owned()),
            port: port.unwrap_or("8080".to_owned()),
            logger: Logger::new(),
            routes: HashMap::new(),
            static_dirs: HashMap::new(),
        }
    }

    pub fn register_static_dir(&mut self, url_path: &str, dir_path: Option<&str>) {
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
            self.logger
                .info(&format!("Hosting files from '{}' at {}", dir, route));
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
            Response {
                status_code: 400,
                headers: headers!(("Content-Type", "text/plain")),
                body: Some(b"400 Bad Request".to_vec()),
            }
            .send(&mut stream)
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

        // If not found in static_dirs, try to match in routes
        if let Some(route) = self.routes.get(&route_index) {
            route(request).await.send(&mut stream).await;
            return;
        }

        // If no route is found, return 404 Not Found
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
        Response {
            status_code: 400,
            headers: headers!(("Content-Type", "text/plain")),
            body: Some(b"404 Not Found".to_vec()),
        }
        .send(&mut stream)
        .await;
    }

    async fn provide_static_dir(
        &self,
        // request: Request,
        dir_path: PathBuf,
        relative_path: String,
    ) -> Response {
        let file_path = dir_path.clone();
        let mut file_path = file_path.join(relative_path.trim_start_matches('/'));
        if file_path.is_dir().await || relative_path.is_empty() || relative_path.ends_with("/") {
            file_path.push("index.html");
        }

        FileResponse {
            status_code: 200,
            headers: None,
            file_path: file_path.to_string_lossy().to_string(),
        }
        .into_response()
        .await
    }
}
