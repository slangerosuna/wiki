# Database Coverage Plan

## Objective
Document scenarios required to validate `Database::login()` behavior, especially `LoginResult::NeedsVerification` and timestamp handling.

## Action Items
- Seed users with adjustable `privileges_last_updated` values.
- Capture expectations for `verify_privilege()` invocation and resulting privilege states.
- Ensure probe records include `privileges`, `user_id`, and optional Patreon identifiers for downstream workflows.
- Leverage `wiki::db::testing::VerificationProbe` to observe verification calls without altering production logic.

## Notes
- Ensure tests operate on isolated SQLite files via temporary directories.
- Use direct `UPDATE` statements through `rusqlite::Connection` to back-date timestamps safely within tests.
