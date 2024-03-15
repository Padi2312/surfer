<p align="center">
  <img src="./docs/surfer_logo.png" alt="Surfer Logo" width="200">
</p>

<h1 align="center">Surfer</h1>

<p align="center">
  <strong>A lightweight, asynchronous backend framework for Rust with Rust</strong>
</p>

It's a <span style="font-size: 7px;">simple</span>, <span style="font-size: 12px;">lightweight</span> and asynchronous backend framework for Rust. It's built on top of `async-std` and provides ~~easy~~ route registration and handling of HTTP requests. It also provides built-in response structs for response creation and JSON response support for structs with Serialize and Deserialize implemented.

## 🚀 Features
- Asynchronous handling of HTTP requests (using async-std)
- Easy route registration with the `route!` macro
- Built-in response structs for easy response creation
- JSON response support for structs with Serialize and Deserialize implemented
- Use the `#[surfer_launch]` macro ~~to start the server~~ to not have to write `#[async_std::main]` (internally it's the same thing :D)

## 📦 Installation

Clone the repository and add the following to your `Cargo.toml`:

```toml
[dependencies]
surfer = "0.3.1"
```

## 📚 Example Usage
```rust
extern crate surfer;

use serde_json::json;
use surfer::request::Method::GET;
use surfer::request::Request;
use surfer::response::json_response::JsonResponse;
use surfer::response::{IntoResponse, Response};
use surfer::route;
use surfer::server::Server;
use surfer_macros::surfer_launch;

async fn index(_: Request) -> Response {
    let json_obj = json!({
        "message": "Hello, Surfer!"
    });
    JsonResponse {
        status_code: 200,
        headers: None,
        body: json_obj,
    }
    .into_response()
    .await
}

#[surfer_launch]
async fn main() {
    let mut server = Server::new(None, None);
    server.register_route(route!(GET, "/", index));
    server.listen().await;
}
```

## 📖 Documentation
For more detailed documentation, get known to the source code 🫠