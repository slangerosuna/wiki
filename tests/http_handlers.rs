use axum::body::{Body, to_bytes};
use axum::http::{header, Request, StatusCode};
use serde_json::{json, Value};
use tokio::time::{timeout, Duration};
use tower::ServiceExt;

#[tokio::test]
async fn app_router_serves_docs_bootstrap_without_auth() {
    with_timeout(async {
        call(
            Request::builder()
                .uri("/docs/apples")
                .body(Body::empty())
                .expect("failed to build request"),
        )
        .await
    })
    .await;

    let response = with_timeout(async {
        call(
            Request::builder()
                .uri("/docs/apples")
                .body(Body::empty())
                .expect("failed to build request"),
        )
        .await
    })
    .await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), 1 << 20)
        .await
        .expect("read body");
    let body = String::from_utf8(body.to_vec()).expect("body utf8");

    assert!(body.contains("redirectToLogin"));
    assert!(body.contains("encodeURIComponent"));
}

#[tokio::test]
async fn register_returns_token_and_privileges() {
    with_timeout(async {
        let username = unique_username("register-success");
        let password = "secret";

        let response = register(&username, password).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_json_response(&response);

        let body = to_body_json(response).await;
        assert!(body.get("token").and_then(Value::as_str).unwrap().len() > 0);
        assert_eq!(body.get("privileges").and_then(Value::as_i64), Some(1));
    })
    .await;
}

#[tokio::test]
async fn login_returns_token_for_valid_credentials() {
    with_timeout(async {
        let username = unique_username("login-success");
        let password = "hunter2";
        let bad_password = "wrong";

        let register_resp = register(&username, password).await;
        assert_eq!(register_resp.status(), StatusCode::OK);

        let response = login(&username, password).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_json_response(&response);
        let body = to_body_json(response).await;
        assert!(body.get("token").and_then(Value::as_str).unwrap().len() > 0);
        assert_eq!(body.get("privileges").and_then(Value::as_i64), Some(1));

        let bad_login = login(&username, bad_password).await;
        assert_eq!(bad_login.status(), StatusCode::UNAUTHORIZED);
    })
    .await;
}

#[tokio::test]
async fn register_duplicate_username_returns_error() {
    with_timeout(async {
        let username = unique_username("register-duplicate");
        let password = "password";

        let first = register(&username, password).await;
        assert_eq!(first.status(), StatusCode::OK);

        let duplicate = register(&username, password).await;
        assert_eq!(duplicate.status(), StatusCode::INTERNAL_SERVER_ERROR);
    })
    .await;
}

fn unique_username(prefix: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos();
    format!("{}-{}", prefix, nanos)
}

async fn register(username: &str, password: &str) -> axum::response::Response {
    let body = json!({
        "username": username,
        "password": password,
    })
    .to_string();

    call(
        Request::builder()
            .method("POST")
            .uri("/api/register")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .expect("register request"),
    )
    .await
}

async fn login(username: &str, password: &str) -> axum::response::Response {
    let body = json!({
        "username": username,
        "password": password,
    })
    .to_string();

    call(
        Request::builder()
            .method("POST")
            .uri("/api/login")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .expect("login request"),
    )
    .await
}

fn assert_json_response(response: &axum::response::Response) {
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .expect("missing content-type header")
        .to_str()
        .expect("invalid content-type header");
    assert!(content_type.starts_with("application/json"));
}

async fn to_body_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), 1 << 20)
        .await
        .expect("read body bytes");
    serde_json::from_slice(&bytes).expect("parse json body")
}

fn app_router() -> axum::Router {
    wiki::app::router()
}

async fn call(request: Request<Body>) -> axum::response::Response {
    let router = app_router();
    timeout(Duration::from_secs(1), router.oneshot(request))
        .await
        .expect("router call exceeded 1s")
        .expect("router call failed")
}

async fn with_timeout<F, T>(future: F) -> T
where
    F: std::future::Future<Output = T>,
{
    timeout(Duration::from_secs(1), future)
        .await
        .expect("test step exceeded 1s")
}
