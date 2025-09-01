use axum::Router;
use lazy_static::lazy_static;
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::{net::TcpListener, sync::mpsc::Receiver};
use tower_http::services::ServeDir;

pub mod db;
pub mod user;
pub mod docs;

use db::Database;
use docs::ServeDocs;

lazy_static! {
    pub static ref DB: Database = Database::new("db.sqlite").unwrap();
}

pub const SECRET_KEY: &'static [u8] = include_bytes!("../secret_key");

#[tokio::main]
async fn main() {
    let (tx, rx) = tokio::sync::mpsc::channel(1);

    tokio::spawn(async move {
        let mut input = String::new();
        let mut reader = BufReader::new(tokio::io::stdin());
        while let Ok(n) = reader.read_line(&mut input).await {
            if n == 0 {
                break; // EOF (I don't think this is possible)
            }

            {
                let input = input.trim();

                if input == "exit" || input == "quit" {
                    tx.send(()).await.unwrap();
                    break;
                }
            }

            input.clear();
        }
    });

    let api_routes = Router::new()
        .route("/api/login", axum::routing::post(user::login_handler))
        .route("/api/register", axum::routing::post(user::register_handler));

    let static_files = ServeDir::new("frontend");

    let app = Router::new()
        .merge(api_routes)
        .nest_service("/docs", ServeDocs::new("docs"))
        .fallback_service(static_files);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running at http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal(rx))
        .await
        .unwrap();

    DB.close().await;
}

async fn shutdown_signal(mut rx: Receiver<()>) {
    rx.recv().await.expect("Sender mysteriously dropped");

    println!("\nReceived shutdown signal, shutting down gracefully...");
}
