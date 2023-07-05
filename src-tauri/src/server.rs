use crate::listeners::create_listeners;
use axum::{response::Html, routing::get, Json, Router};
use hyperlocal::UnixServerExt;

async fn hello_world() -> Json<String> {
    Json("Hello, World!".into())
}

pub fn build_server() -> Result<(), String> {
    // Define your API routes
    let app: Router<Json<String>> = Router::new().route("/", get(hello_world));
    Ok(())
}
