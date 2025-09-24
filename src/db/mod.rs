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

use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{PasswordHash, SaltString, rand_core::OsRng};
use rusqlite::{Connection, Result, params};
use std::sync::OnceLock;
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
        password: String,
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
                    username TEXT NOT NULL UNIQUE,
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
                            let result = conn
                                .execute(
                                    "UPDATE users SET privileges = ?1, privileges_last_updated = CURRENT_TIMESTAMP WHERE id = ?2",
                                    params![privileges, user_id],
                                )
                                .map(|_| ());
                            let _ = resp.send(result);
                        }
                        DbRequest::Login { username, password, resp } => {
                            let mut stmt = match conn.prepare(
                                "SELECT password, privileges, privileges_last_updated, id, patreon_id, patreon_refresh_token FROM users WHERE username = ?1"
                            ) {
                                Ok(stmt) => stmt,
                                Err(e) => {
                                    let _ = resp.send(Err(e));
                                    continue;
                                }
                            };

                            let mut rows = match stmt.query(params![username]) {
                                Ok(rows) => rows,
                                Err(e) => {
                                    let _ = resp.send(Err(e));
                                    continue;
                                }
                            };

                            use chrono::{Duration, NaiveDateTime, Utc};

                            let result: Result<LoginResult> = match rows.next() {
                                Ok(Some(row)) => {
                                    let stored_password: String = row.get(0)?;
                                    let login_allowed = PasswordHash::new(&stored_password)
                                        .ok()
                                        .and_then(|hash| {
                                            Argon2::default()
                                                .verify_password(password.as_bytes(), &hash)
                                                .ok()
                                        })
                                        .is_some();

                                    if !login_allowed {
                                        return Ok(LoginResult::Privileges(None));
                                    }

                                    let privileges: i32 = row.get(1)?;
                                    let last_updated_str: String = row.get(2)?;
                                    let user_id: i32 = row.get(3)?;
                                    let patreon_id: Option<String> = row.get(4)?;
                                    let patreon_refresh_token: Option<String> = row.get(5)?;

                                    let needs_verify = if privileges != 0 && privileges != 1 {
                                        if let Ok(last_updated) =
                                            NaiveDateTime::parse_from_str(&last_updated_str, "%Y-%m-%d %H:%M:%S")
                                        {
                                            let now = Utc::now().naive_utc();
                        }}
                    }
                }
            });
        });
        Ok(Database { tx })
    }

    pub async fn add_user(
        &self,
        username: &str,
        password: &str,
        privileges: i32,
    ) -> Result<()> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|err| rusqlite::Error::InvalidParameterName(err.to_string()))?
            .to_string();

        let (resp_tx, resp_rx) = oneshot::channel();

        let req = DbRequest::AddUser {
            username: username.to_string(),
            password_hash,
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

    pub async fn login(&self, username: &str, password: &str) -> Result<Option<i32>> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let req = DbRequest::Login {
            username: username.to_string(),
            password: password.to_string(),
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
    testing::with_verification_probe(|probe| {
        probe.record_call(testing::VerificationCall {
            privileges,
            user_id,
            patreon_id: patreon_id.clone(),
            patreon_refresh_token: patreon_refresh_token.clone(),
        });
    });
    // TODO: Implement verification logic using patreon oauth
    let _ = (patreon_id, patreon_refresh_token, user_id, privileges);
    privileges
}

pub mod testing {
    use super::{Connection, OnceLock};
    use chrono::{Duration, Utc};
    use rusqlite::params;
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    };

    #[derive(Clone)]
    pub struct VerificationProbe {
        called: Arc<AtomicBool>,
        last_call: Arc<Mutex<Option<VerificationCall>>>,
    }

    impl Default for VerificationProbe {
        fn default() -> Self {
            Self {
                called: Arc::new(AtomicBool::new(false)),
                last_call: Arc::new(Mutex::new(None)),
            }
        }
    }

    impl VerificationProbe {
        pub fn was_called(&self) -> bool {
            self.called.load(Ordering::SeqCst)
        }

        pub fn last_call(&self) -> Option<VerificationCall> {
            self.last_call.lock().expect("probe mutex poisoned").clone()
        }

        pub(crate) fn record_call(&self, call: VerificationCall) {
            self.called.store(true, Ordering::SeqCst);
            *self.last_call.lock().expect("probe mutex poisoned") = Some(call);
        }
    }

    /// Back-date a user's `privileges_last_updated` field by the provided duration while
    /// clamping negative durations to zero to avoid advancing the timestamp.
    pub fn backdate_privileges(
        conn: &Connection,
        username: &str,
        delta: Duration,
    ) -> rusqlite::Result<()> {
        let clamped = if delta < Duration::zero() {
            Duration::zero()
        } else {
            delta
        };
        let target_timestamp = (Utc::now() - clamped)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        conn.execute(
            "UPDATE users SET privileges_last_updated = ?1 WHERE username = ?2",
            params![target_timestamp, username],
        )
        .map(|_| ())
    }

    static VERIFICATION_PROBE: OnceLock<Mutex<Option<VerificationProbe>>> = OnceLock::new();

    fn probe_slot() -> &'static Mutex<Option<VerificationProbe>> {
        VERIFICATION_PROBE.get_or_init(|| Mutex::new(None))
    }

    pub fn set_verification_probe(probe: VerificationProbe) {
        *probe_slot().lock().expect("probe mutex poisoned") = Some(probe);
    }

    pub fn clear_verification_probe() {
        *probe_slot().lock().expect("probe mutex poisoned") = None;
    }

    pub(crate) fn with_verification_probe<F: FnOnce(&VerificationProbe)>(f: F) {
        if let Some(probe) = probe_slot().lock().expect("probe mutex poisoned").clone() {
            f(&probe);
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct VerificationCall {
        pub privileges: i32,
        pub user_id: i32,
        pub patreon_id: Option<String>,
        pub patreon_refresh_token: Option<String>,
    }
}
