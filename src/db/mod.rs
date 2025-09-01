#[derive(Debug)]
pub enum LoginResult {
    Privileges(Option<i32>),
    NeedsVerification {
        privileges: i32,
        user_id: i32,
        patreon_id: Option<String>,
        patreon_refresh_token: Option<String>,
    },
}

use rusqlite::{Connection, Result, params};
use tokio::sync::{mpsc, oneshot};
use tokio::task;

#[derive(Debug)]
pub enum DbRequest {
    AddUser {
        username: String,
        password_hash: String,
        privileges: i32,
        resp: oneshot::Sender<Result<()>>,
    },
    SetUserPrivileges {
        user_id: i32,
        privileges: i32,
        resp: oneshot::Sender<Result<()>>,
    },
    Login {
        username: String,
        password_hash: String,
        resp: oneshot::Sender<Result<LoginResult>>,
    },
    Close,
}

#[derive(Clone)]
pub struct Database {
    tx: mpsc::Sender<DbRequest>,
}

impl Database {
    pub async fn close(&self) {
        println!("Closing database connection...");
        self.tx.send(DbRequest::Close).await.unwrap();
    }

    pub fn new(db_path: &str) -> Result<Self> {
        let (tx, mut rx) = mpsc::channel::<DbRequest>(32);
        let db_path = db_path.to_string();

        let _ = task::spawn_blocking(move || {
            println!("Database connection opening...");
            let conn = Connection::open(&db_path).expect("Failed to open DB");
            conn.execute(
                "CREATE TABLE IF NOT EXISTS users (
                    id INTEGER PRIMARY KEY,
                    username TEXT NOT NULL,
                    password TEXT NOT NULL,
                    privileges INTEGER NOT NULL,
                    privileges_last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    patreon_id TEXT,
                    patreon_refresh_token TEXT
                )",
                [],
            )
            .expect("Failed to create users table");

            let rt = tokio::runtime::Handle::current();
            rt.block_on(async move {
                while let Some(req) = rx.recv().await {
                    match req {
                        DbRequest::Close => {
                            rx.close();

                            println!("Database connection closed.");
                            break;
                        }
                        DbRequest::AddUser { username, password_hash, privileges, resp } => {
                            let result = conn.execute(
                                "INSERT INTO users (username, password, privileges) VALUES (?1, ?2, ?3)",
                                params![username, password_hash, privileges],
                            ).map(|_| ());
                            let _ = resp.send(result);
                        }
                        DbRequest::SetUserPrivileges { user_id, privileges, resp } => {
                            let result = conn.execute(
                                "UPDATE users SET privileges = ?1, privileges_last_updated = CURRENT_TIMESTAMP WHERE id = ?2",
                                params![privileges, user_id],
                            ).map(|_| ());
                            let _ = resp.send(result);
                        }
                        DbRequest::Login { username, password_hash, resp } => {
                            let mut stmt = match conn.prepare(
                                "SELECT privileges, privileges_last_updated, id, patreon_id, patreon_refresh_token FROM users WHERE username = ?1 AND password = ?2"
                            ) {
                                Ok(stmt) => stmt,
                                Err(e) => {
                                    let _ = resp.send(Err(e));
                                    continue;
                                }
                            };
                            let mut rows = match stmt.query(params![username, password_hash]) {
                                Ok(rows) => rows,
                                Err(e) => {
                                    let _ = resp.send(Err(e));
                                    continue;
                                }
                            };
                            use chrono::{NaiveDateTime, Utc, Duration};
                            let result = match rows.next() {
                                Ok(Some(row)) => {
                                    let privileges: i32 = match row.get(0) {
                                        Ok(p) => p,
                                        Err(e) => return { let _ = resp.send(Err(e)); },
                                    };
                                    let last_updated_str: String = match row.get(1) {
                                        Ok(s) => s,
                                        Err(e) => return { let _ = resp.send(Err(e)); },
                                    };
                                    let user_id: i32 = match row.get(2) {
                                        Ok(id) => id,
                                        Err(e) => return { let _ = resp.send(Err(e)); },
                                    };
                                    let patreon_id: Option<String> = match row.get(3) {
                                        Ok(d) => d,
                                        Err(e) => return { let _ = resp.send(Err(e)); },
                                    };
                                    let patreon_refresh_token: Option<String> = match row.get(4) {
                                        Ok(t) => t,
                                        Err(e) => return { let _ = resp.send(Err(e)); },
                                    };
                                    // Parse timestamp and check if > 30 days
                                    let needs_verify = if privileges != 0 && privileges != 1 {
                                        if let Ok(last_updated) = NaiveDateTime::parse_from_str(&last_updated_str, "%Y-%m-%d %H:%M:%S") {
                                            let now = Utc::now().naive_utc();
                                            now.signed_duration_since(last_updated) > Duration::days(30)
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    };
                                    if needs_verify {
                                        Ok(LoginResult::NeedsVerification {
                                            privileges,
                                            user_id,
                                            patreon_id,
                                            patreon_refresh_token,
                                        })
                                    } else {
                                        Ok(LoginResult::Privileges(Some(privileges)))
                                    }
                                }
                                Ok(None) => Ok(LoginResult::Privileges(None)),
                                Err(e) => Err(e),
                            };
                            let _ = resp.send(result);
                        }
                    }
                }
            });
        });
        Ok(Database { tx })
    }

    pub async fn add_user(
        &self,
        username: &str,
        password_hash: &str,
        privileges: i32,
    ) -> Result<()> {
        let (resp_tx, resp_rx) = oneshot::channel();
        
        let req = DbRequest::AddUser {
            username: username.to_string(),
            password_hash: password_hash.to_string(),
            privileges,
            resp: resp_tx,
        };

        self.tx
            .send(req)
            .await
            .expect("Failed to send AddUser request");

        resp_rx.await.expect("DB thread panicked")
    }

    pub async fn set_user_privileges(&self, user_id: i32, privileges: i32) -> Result<()> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let req = DbRequest::SetUserPrivileges {
            user_id,
            privileges,
            resp: resp_tx,
        };

        self.tx
            .send(req)
            .await
            .expect("Failed to send SetUserPrivileges request");

        resp_rx.await.expect("DB thread panicked")
    }

    pub async fn login(&self, username: &str, password_hash: &str) -> Result<Option<i32>> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let req = DbRequest::Login {
            username: username.to_string(),
            password_hash: password_hash.to_string(),
            resp: resp_tx,
        };

        self.tx
            .send(req)
            .await
            .expect("Failed to send Login request");

        match resp_rx.await.expect("DB thread panicked")? {
            LoginResult::Privileges(privs) => Ok(privs),
            LoginResult::NeedsVerification {
                privileges,
                user_id,
                patreon_id,
                patreon_refresh_token,
            } => {
                verify_privilege(privileges, user_id, patreon_id, patreon_refresh_token);
                Ok(Some(privileges))
            }
        }
    }
}

fn verify_privilege(
    privileges: i32,
    user_id: i32,
    patreon_id: Option<String>,
    patreon_refresh_token: Option<String>,
) -> i32 {
    // TODO: Implement verification logic using patreon oauth
    let _ = (patreon_id, patreon_refresh_token, user_id, privileges);
    privileges
}
