use axum::Router;
use lazy_static::lazy_static;
use std::net::SocketAddr;
use tokio::{net::TcpListener, signal::ctrl_c};
use tower_http::services::ServeDir;

pub mod db;
pub mod user;

use db::Database;

lazy_static! {
    pub static ref DB: Database = Database::new("db.sqlite").unwrap();
}

pub const SECRET_KEY: &'static [u8] = include_bytes!("../secret_key");

#[tokio::main]
async fn main() {
    let api_routes = Router::new().route("/api/login", axum::routing::post(user::login_handler));

    let static_files = ServeDir::new("frontend");

    let app = Router::new()
        .merge(api_routes)
        .fallback_service(static_files);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running at http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    DB.close().await;
}

async fn shutdown_signal() {
    ctrl_c().await.expect("failed to install Ctrl+C handler");

    println!("\nReceived Ctrl+C, shutting down gracefully...");
}
