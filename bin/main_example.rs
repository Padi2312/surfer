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

#[cfg(not(release))]
#[surfer_launch]
async fn main() {
    println!("Testing the 'surfer' library...");
    let mut server = Server::new(None, None);
    server.register_route(route!(GET, "/", index));
    server.listen().await;
}
