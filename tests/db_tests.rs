use std::path::PathBuf;

use chrono::{Duration, Utc};
use rusqlite::{Connection, params};
use tempfile::tempdir;
use wiki::db::Database;

fn temp_db_path() -> (tempfile::TempDir, PathBuf) {
    let dir = tempdir().expect("failed to create temp dir");
    let path = dir.path().join("wiki-tests.sqlite");
    (dir, path)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn add_user_and_login_succeeds() {
    let (_dir, path) = temp_db_path();
    let db = Database::new(path.to_str().unwrap()).expect("failed to create db");

    db.add_user("alice", "password", 1)
        .await
        .expect("add_user failed");

    let privileges = db.login("alice", "password").await.expect("login failed");
    assert_eq!(privileges, Some(1));

    db.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn login_with_invalid_credentials_returns_none() {
    let (_dir, path) = temp_db_path();
    let db = Database::new(path.to_str().unwrap()).expect("failed to create db");

    db.add_user("alice", "password", 1)
        .await
        .expect("add_user failed");

    let privileges = db
        .login("alice", "wrong-password")
        .await
        .expect("login failed");
    assert!(privileges.is_none());

    db.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn set_user_privileges_updates_role() {
    let (_dir, path) = temp_db_path();
    let db = Database::new(path.to_str().unwrap()).expect("failed to create db");

    db.add_user("bob", "hunter2", 1)
        .await
        .expect("add_user failed");

    let conn = Connection::open(&path).expect("open connection");
    let user_id: i32 = conn
        .query_row(
            "SELECT id FROM users WHERE username = ?1",
            [&"bob"],
            |row| row.get(0),
        )
        .expect("query user id");

    db.set_user_privileges(user_id, 2)
        .await
        .expect("set_user_privileges failed");

    let privileges = db.login("bob", "hunter2").await.expect("login failed");
    assert_eq!(privileges, Some(2));

    db.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn login_with_stale_privileges_triggers_verification() {
    let (_dir, path) = temp_db_path();
    let db = Database::new(path.to_str().unwrap()).expect("failed to create db");

    let probe = wiki::db::testing::VerificationProbe::default();
    wiki::db::testing::set_verification_probe(probe.clone());

    db.add_user("carol", "password", 5)
        .await
        .expect("add_user failed");

    let conn = Connection::open(&path).expect("open connection");
    let user_id: i32 = conn
        .query_row(
            "SELECT id FROM users WHERE username = ?1",
            [&"carol"],
            |row| row.get(0),
        )
        .expect("query user id");
    let stale = (Utc::now() - Duration::days(31)).format("%Y-%m-%d %H:%M:%S");
    conn.execute(
        "UPDATE users SET privileges_last_updated = ?1, patreon_id = ?2, patreon_refresh_token = ?3 WHERE username = ?4",
        params![stale.to_string(), "patreon-carol", "refresh-token", "carol"],
    )
    .expect("update timestamp and patreon fields");

    let privileges = db.login("carol", "password").await.expect("login failed");
    assert_eq!(privileges, Some(5));

    assert!(probe.was_called());
    let call = probe.last_call().expect("probe recorded call");
    assert_eq!(call.privileges, 5);
    assert_eq!(call.user_id, user_id);
    assert_eq!(call.patreon_id.as_deref(), Some("patreon-carol"));
    assert_eq!(call.patreon_refresh_token.as_deref(), Some("refresh-token"));

    wiki::db::testing::clear_verification_probe();
    db.close().await;
}
