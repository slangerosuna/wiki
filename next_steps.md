# Legend

- **[ ]**: Not started
- **[~]**: In progress
- **[x]**: Completed

# Next Steps

## Database Coverage Enhancements
- **Exercise `LoginResult::NeedsVerification`**: Seed a user whose `privileges_last_updated` exceeds 30 days and ensure `verify_privilege()` is called via `Database::login()`. Validate that the branch returns the expected privileges and triggers follow-up behavior. **[ ]**
- **Back-date privilege timestamps safely**: Add a helper to adjust `privileges_last_updated` using `rusqlite::Connection` so each test can control the verification window without race conditions. **[ ]**
- **Ensure that login uses crytographically secure methods**: **[ ]**

## Rewrite to improve existing code
- **Rewrite everything that involves `src/docs/pull_jwt_or_forward_to_login.js` as it is absolutely way too janky** **[ ]**

## HTTP Handler End-to-End Tests
- **Spin up the Axum router**: Instantiate the `Router` from `src/main.rs` using components exported through `src/lib.rs`, and run requests against `/api/register` and `/api/login` with `tower::ServiceExt::oneshot`. **[ ]**
- **Assert response semantics**: Confirm successful registration returns a token + privileges payload, duplicate usernames surface `500`, and invalid credentials return `401`. **[ ]**
- **Validate side effects**: After registering, query the temporary database to ensure the user row exists with the correct privilege level. **[ ]**

## `ServeDocs` Integration Coverage
- **Test edit mode rendering**: Issue a request to `/docs/<path>?edit` and verify the form contains the current markdown contents. **[ ]**
- **Confirm 404 behavior**: Request a non-existent document and assert the `404 Not Found` response body is emitted. **[ ]**
- **Privilege-gated display**: Send requests with JWTs representing different privilege levels and ensure restricted sections are included or omitted as expected. **[ ]**

## Shared Test Utilities
- **Centralize fixtures**: Move repeated helpers (temporary DB creation, JWT generation) into `tests/util.rs` or a `tests/support/` module for reuse across suites. **[ ]**
- **Facilitate JWT issuance**: Provide a small helper that signs tokens with `SECRET_KEY` so privilege-specific tests remain concise. **[ ]**

## Coverage Tracking
- **Introduce coverage tooling**: Evaluate `cargo tarpaulin` (or equivalent) integration to quantify test coverage and surface gaps over time. **[ ]**
- **Automate in CI**: Once coverage tooling is validated locally, wire it into the project’s CI to prevent regressions in test breadth. **[ ]**
