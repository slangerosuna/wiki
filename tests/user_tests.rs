use jsonwebtoken::{EncodingKey, Header, encode};
use serde::Serialize;
use wiki::SECRET_KEY;
use wiki::user::{get_current_timestamp, get_jwt_perms};

#[test]
fn guest_token_maps_to_basic_privileges() {
    assert_eq!(get_jwt_perms("guest"), Some(1));
}

#[test]
fn invalid_token_returns_none() {
    assert_eq!(get_jwt_perms("not-a-token"), None);
}

#[test]
fn valid_token_returns_encoded_privileges() {
    #[derive(Serialize)]
    struct Claims {
        sub: String,
        privileges: i32,
        exp: u64,
    }

    let claims = Claims {
        sub: "alice".into(),
        privileges: 3,
        exp: get_current_timestamp() + 3600,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET_KEY),
    )
    .expect("failed to encode token");

    assert_eq!(get_jwt_perms(&token), Some(3));
}
