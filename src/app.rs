use axum::Router;
use axum::routing::post;
use tower_http::services::ServeDir;

use crate::{docs::ServeDocs, user};

pub fn router() -> Router {
    let api_routes = Router::new()
        .route("/api/login", post(user::login_handler))
        .route("/api/register", post(user::register_handler));

    let static_files = ServeDir::new("frontend");

    Router::new()
        .merge(api_routes)
        .nest_service("/docs", ServeDocs::new("docs"))
        .fallback_service(static_files)
}
