use lazy_static::lazy_static;

pub mod app;
pub mod db;
pub mod docs;
pub mod user;

use db::Database;

lazy_static! {
    pub static ref DB: Database = Database::new("db.sqlite").unwrap();
}

pub const SECRET_KEY: &[u8] = include_bytes!("../secret_key");
