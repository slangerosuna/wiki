use axum::body::Body;
use axum::http::request::Request;
use serde_json;
use std::future::Future;
use std::pin::Pin;
use tower_service::Service;

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

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let path = self.path.clone();
        Box::pin(async move {
            let permissions;
            if let Some(jwt) = req.headers().get("Authorization") {
                let jwt = jwt
                    .to_str()
                    .unwrap_or("")
                    .strip_prefix("Bearer ")
                    .unwrap_or("");
                permissions = crate::user::get_jwt_perms(jwt).unwrap_or(1);
            } else {
                let redirect_target = req
                    .uri()
                    .path_and_query()
                    .map(|pq| pq.as_str().to_string())
                    .unwrap_or_else(|| req.uri().path().to_string());
                let redirect_literal = serde_json::to_string(&redirect_target).unwrap();
                let bootstrap = include_str!("pull_jwt_or_forward_to_login.js")
                    .replace("__REDIRECT_TARGET__", &redirect_literal);

                let html = format!(
                    "<!doctype html><html><head><meta charset=\"utf-8\"><title>Loading documentation…</title></head><body><div id=\"docs-bootstrap\"></div><script>{}</script></body></html>",
                    bootstrap
                );

                return Ok(axum::response::Response::builder()
                    .status(axum::http::StatusCode::OK)
                    .body(Body::from(html))
                    .unwrap());
            }

            let uri = req.uri();
            if uri.query().map(|q| q.contains("edit")).unwrap_or(false) {
                let uri = uri.path();
                let doc_path = format!("{}{}.md", path, uri);
                let contents = tokio::fs::read_to_string(&doc_path)
                    .await
                    .unwrap_or_default();

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

            let html = parse_markdown(&doc, permissions);

            let css = include_str!("styles.css");
            let js = include_str!("main.js");

            let html = format!(include_str!("format.html"), css, js, html);

            Ok(axum::response::Response::builder()
                .status(200)
                .body(Body::from(html))
                .unwrap())
        })
    }
}

pub fn parse_markdown(doc: &str, permissions: i32) -> String {
    let mut sections = Vec::new();
    let mut current_section = String::new();
    let mut skip_section = false;

    for line in doc.lines() {
        if line.starts_with('!') {
            if !current_section.is_empty() && !skip_section {
                sections.push(std::mem::take(&mut current_section));
            } else {
                current_section.clear();
            }

            let required_level = line[1..]
                .chars()
                .next()
                .and_then(|c| c.to_digit(10))
                .unwrap_or(1) as i32;

            skip_section = permissions != 0 && permissions < required_level;
        } else {
            current_section.push_str(line);
            current_section.push('\n');
        }
    }
    if !current_section.is_empty() && !skip_section {
        sections.push(current_section);
    }

    let page = sections
        .into_iter()
        .map(|section| comrak::markdown_to_html(&section, &comrak::Options::default()))
        .collect::<Vec<_>>()
        .join("\n");

    if page.trim().is_empty() {
        "Page requires higher privileges, try logging in.".into()
    } else {
        page
    }
}
