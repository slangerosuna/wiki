use std::future::Future;
use std::pin::Pin;
use tower_service::Service;
use axum::http::request::Request;
use axum::body::Body;

#[derive(Clone)]
pub struct ServeDocs {
    path: String,
}

impl ServeDocs {
    pub fn new(path: &str) -> Self {
        ServeDocs { path: path.into() }
    }
}

impl Service<Request<Body>> for ServeDocs {
    type Response = axum::response::Response;
    type Error = std::convert::Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let path = self.path.clone();
        Box::pin(async move {
            let uri = req.uri();
            // Check to see if the uri ends in ?edit
            if uri.query().map(|q| q.contains("edit")).unwrap_or(false) {
                let uri = uri.path();
                // Serve an editor interface for the markdown, creating a new file if it doesn't exist already
                let doc_path = format!("{}{}.md", path, uri);
                let contents = tokio::fs::read_to_string(&doc_path).await.unwrap_or_default();

                // Serve an editor interface for the markdown
                let response = axum::response::Response::builder()
                    .status(200)
                    .body(Body::from(format!(
                        "<html><body><form method=\"post\" action=\"/docs{}?edit\"><textarea name=\"content\" rows=\"20\" cols=\"80\">{}</textarea><br><button type=\"submit\">Save</button></form></body></html>",
                        uri, contents
                    )))
                    .unwrap();

                return Ok(response);
            }
            let uri = req.uri().path();
            let doc_path = format!("{}{}.md", path, uri);

            let doc = match tokio::fs::read_to_string(&doc_path).await {
                Ok(doc) => doc,
                Err(_) => {
                    return Ok(axum::response::Response::builder()
                        .status(axum::http::StatusCode::NOT_FOUND)
                        .body(Body::from("Not found"))
                        .unwrap());
                }
            };

            let html = comrak::markdown_to_html(&doc, &comrak::Options::default());
            let css = tokio::fs::read_to_string("docs/styles.css").await.unwrap_or_default();

            let html = format!(
                "<html><head><style>{}</style></head><body>{}</body></html>",
                css, html
            );

            Ok(axum::response::Response::builder()
                .status(200)
                .body(Body::from(html))
                .unwrap())
        })
    }
}