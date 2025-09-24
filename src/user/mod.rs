use axum::{Json, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::SECRET_KEY;

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

pub async fn login_handler(Json(payload): Json<LoginRequest>) -> impl IntoResponse {
    let Ok(Some(privilege)) = crate::DB
        .login(payload.username.as_str(), payload.password.as_str())
        .await
    else {
        return (
            axum::http::StatusCode::UNAUTHORIZED,
            "Invalid username or password",
        )
            .into_response();
    };

    let auth_token = create_jwt(payload.username.as_str(), privilege).unwrap();

    #[derive(Serialize)]
    struct LoginResponse {
        token: String,
        privileges: i32,
    }

    let response = LoginResponse {
        token: auth_token,
        privileges: privilege,
    };

    let response = serde_json::to_string(&response).unwrap();

    (axum::http::StatusCode::OK, response).into_response()
}

pub async fn register_handler(Json(payload): Json<LoginRequest>) -> impl IntoResponse {
    let Ok(_) = crate::DB
        .add_user(
            payload.username.as_str(),
            payload.password.as_str(),
            if payload.username == "root" { 0 } else { 1 },
        )
        .await
    else {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to register user",
        )
            .into_response();
    };

    let Ok(Some(privilege)) = crate::DB
        .login(payload.username.as_str(), payload.password.as_str())
        .await
    else {
        return (
            axum::http::StatusCode::UNAUTHORIZED,
            "Invalid username or password",
        )
            .into_response();
    };

    let auth_token = create_jwt(payload.username.as_str(), privilege).unwrap();

    #[derive(Serialize)]
    struct LoginResponse {
        token: String,
        privileges: i32,
    }

    let response = LoginResponse {
        token: auth_token,
        privileges: privilege,
    };

    let response = serde_json::to_string(&response).unwrap();

    (axum::http::StatusCode::OK, response).into_response()
}

#[derive(Deserialize, Serialize)]
struct JwtClaims {
    sub: String,
    privileges: i32,
    exp: u64,
}

fn create_jwt(username: &str, privileges: i32) -> Result<String, jsonwebtoken::errors::Error> {
    use jsonwebtoken::{EncodingKey, Header, encode};

    let expiration = get_current_timestamp() + 24 * 3600; // 24 hours from now

    let claims = JwtClaims {
        sub: username.to_string(),
        privileges,
        exp: expiration,
    };

    let encoding_key = EncodingKey::from_secret(SECRET_KEY);
    encode(&Header::default(), &claims, &encoding_key)
}

pub fn get_jwt_perms(jwt: &str) -> Option<i32> {
    use jsonwebtoken::{DecodingKey, Validation, decode};

    if jwt == "guest" {
        return Some(1);
    }

    let decoding_key = DecodingKey::from_secret(SECRET_KEY);
    let validation = Validation::default();

    match decode::<JwtClaims>(jwt, &decoding_key, &validation) {
        Ok(token_data) => {
            let claims = token_data.claims;
            Some(claims.privileges)
        }
        Err(_) => None,
    }
}

pub fn get_current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
