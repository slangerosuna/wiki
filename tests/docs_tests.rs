use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use tower_service::Service;
use wiki::docs::parse_markdown;

#[test]
fn parse_markdown_respects_privilege_markers() {
    let doc = "!1\nVisible section\n\n!3\nRestricted section\n";

    let rendered = parse_markdown(doc, 1);

    assert!(rendered.contains("<p>Visible section</p>"));
    assert!(!rendered.contains("Restricted section"));
}

#[test]
fn parse_markdown_returns_placeholder_when_everything_hidden() {
    let doc = "!4\nTop secret\n";

    let rendered = parse_markdown(doc, 1);

    assert_eq!(rendered, "Page requires higher privileges, try logging in.");
}

#[tokio::test]
async fn serve_docs_redirects_to_login_without_jwt() {
    let mut service = wiki::docs::ServeDocs::new("docs/");

    let request = Request::builder()
        .uri("/welcome")
        .body(Body::empty())
        .unwrap();

    let response = service.call(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = to_bytes(response.into_body(), 1 << 20).await.unwrap();
    let body = String::from_utf8(body_bytes.to_vec()).unwrap();

    assert!(
        body.contains("redirectToLogin"),
        "helper should be present in bootstrap script"
    );
    assert!(
        body.contains("Bearer"),
        "script should fetch with bearer token"
    );
    assert!(
        body.contains("encodeURIComponent"),
        "script should encode redirect target"
    );
}
