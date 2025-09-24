# Wiki

## Overview
The `wiki` project is a full-stack prototype written in Rust and TypeScript-free static assets. It exposes a small authentication API using `axum`, serves Markdown-based documentation with privilege gates, and falls back to a static portfolio-style frontend. Authentication relies on JWTs signed with a local secret key and user accounts backed by SQLite.

## Features
- **Authentication API**: `src/user/mod.rs` provides `/api/login` and `/api/register` endpoints that hash-free store credentials, mint JWTs, and expose privilege levels in responses.
- **SQLite worker**: `src/db/mod.rs` implements an asynchronous façade around `rusqlite`, spawning a blocking task to serialize SQL work and support test hooks for privilege verification.
- **Privileged docs**: `src/docs/mod.rs` wraps Markdown files beneath `docs/` so sections prefixed with `!<level>` only render for JWTs with sufficient privileges. An `?edit` query renders a simple editing form.
- **Static frontend**: `frontend/` hosts a portfolio shell with dropdown navigation, theme toggles, and a login form (`frontend/login/`) that consumes the API and stores JWTs in `localStorage`.

## Directory tour
- **`src/main.rs`**: Binds routes, nests `ServeDocs`, and serves `frontend/` via `tower_http::services::ServeDir`.
- **`src/lib.rs`**: Exposes module wiring, lazy-initializes the global SQLite-backed `DB`, and embeds the signing `SECRET_KEY`.
- **`src/db/`**: Houses the database dispatcher and `testing` utilities such as `VerificationProbe` and `backdate_privileges()`.
- **`src/docs/`**: Contains the Markdown renderer, HTML template, and client helpers like `pull_jwt_or_forward_to_login.js` for gated views.
- **`frontend/`**: Static HTML/CSS/JS assets for the landing page and login flow.
- **`docs/`**: Markdown content rendered by `ServeDocs`; `docs/plans/` includes project planning notes.
- **`tests/`**: Async integration tests for the database module, JWT helpers, and docs renderer.
- **`next_steps.md`**: Running backlog of enhancement ideas and testing goals.

## Prerequisites
- **Rust toolchain**: Install Rust 1.79+ (edition 2024) via [rustup](https://rustup.rs/).
- **SQLite**: Bundled `rusqlite` uses the system library; most Linux distributions include it by default.
- **Secret key**: Provide a binary file at `secret_key` (for example, 32 random bytes) so JWT encoding/decoding works. This file is ignored by Git; generate it with `openssl rand -out secret_key 32` or an equivalent tool.

## Setup
1. **Install dependencies**: `cargo fetch` downloads the Rust crates specified in `Cargo.toml`.
2. **Create `secret_key`**: Place the secret file at the repository root before running the app or tests.
3. **Prepare docs** *(optional)*: Add Markdown pages under `docs/`. Use lines like `!2` to gate sections to privilege level ≥2.

## Running the app
1. **Start the server**: `cargo run` launches the Axum application on `http://127.0.0.1:3000`.
2. **Static frontend**: Visit `/` for the portfolio shell. The login page lives at `/login/` and writes JWTs to `localStorage`.
3. **Docs browser**: Navigate to `/docs/<page>` (for example, `/docs/apples`). Supply an `Authorization: Bearer <token>` header or visit without a token to trigger the redirect helper. Append `?edit` to load the simple editor form.
4. **Shutdown**: Type `exit` or `quit` on stdin to trigger graceful shutdown; the server also closes the database channel on exit.

## Testing
- **Cargo tests**: Run `cargo test` to execute async database tests, JWT helpers, and Markdown privilege enforcement. Tests create temporary SQLite files and may call `Database::close()` to cleanly stop the worker.
- **Manual verification**: Use the login form to register a user, then access docs requiring elevated privileges to observe gated sections becoming visible.

## Development tips
- **Database schema**: The `users` table lives in `db.sqlite` during development and is created on first run. Remove the file to reset state.
- **Privilege verification hooks**: The `VerificationProbe` in `src/db/mod.rs` lets tests assert that stale privilege records invoke the (stubbed) Patreon verification call.
- **Future work**: See `next_steps.md` for planned coverage improvements, doc serving refinements, and HTTP handler integration tests.

## Project status
This codebase is an early-stage sandbox. Credentials are stored as plaintext, and the OAuth verification flow is stubbed. Treat it as a learning project, not production-ready software.
