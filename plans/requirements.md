# Requirements Tracker

This document mirrors the roadmap in `next_steps.md` and will be updated after each TDD cycle.

## Database Coverage Enhancements

| Item | Status | Test Coverage | Plan Doc | Notes |
| --- | --- | --- | --- | --- |
| Exercise `LoginResult::NeedsVerification` | [x] | `tests/db_tests.rs::login_with_stale_privileges_triggers_verification` | `docs/plans/database_coverage.md` | Stale timestamps trigger `verify_privilege()` via `VerificationProbe`; probe captures privileges/user/patreon metadata while login returns original privileges. |
| Back-date privilege timestamps safely | [x] | `tests/db_tests.rs::login_with_stale_privileges_triggers_verification` | `docs/plans/database_coverage.md` | Shared `wiki::db::testing::backdate_privileges()` clamps negative deltas and updates timestamps for deterministic verification coverage. |

## HTTP Handler End-to-End Tests

| Item | Status | Test Coverage | Plan Doc | Notes |
| --- | --- | --- | --- | --- |
| Spin up the Axum router | [ ] | `tests/http_handlers.rs::router_setup` (planned) | `docs/plans/http_handlers.md` | Build integration harness reusing `wiki::` exports. |
| Assert response semantics | [ ] | `tests/http_handlers.rs::response_semantics` (planned) | `docs/plans/http_handlers.md` | Validate status codes and payload consistency for login/register flows. |
| Validate side effects | [ ] | `tests/http_handlers.rs::side_effects` (planned) | `docs/plans/http_handlers.md` | Confirm DB state reflects registration outcomes. |

## `ServeDocs` Integration Coverage

| Item | Status | Test Coverage | Plan Doc | Notes |
| --- | --- | --- | --- | --- |
| Test edit mode rendering | [ ] | `tests/serve_docs.rs::edit_mode` (planned) | `docs/plans/serve_docs.md` | Ensure form is populated with markdown contents. |
| Confirm 404 behavior | [ ] | `tests/serve_docs.rs::not_found` (planned) | `docs/plans/serve_docs.md` | Missing docs should yield `404 Not Found`. |
| Privilege-gated display | [ ] | `tests/serve_docs.rs::privilege_filtering` (planned) | `docs/plans/serve_docs.md` | JWT privileges govern rendered sections. |

## Shared Test Utilities

| Item | Status | Test Coverage | Plan Doc | Notes |
| --- | --- | --- | --- | --- |
| Centralize fixtures | [ ] | `tests/support/mod.rs` (planned) | `docs/plans/test_utilities.md` | Provide reusable temp DB, JWT, and filesystem helpers. |
| Facilitate JWT issuance | [ ] | `tests/support/jwt.rs` (planned) | `docs/plans/test_utilities.md` | Helper to sign tokens per privilege level. |

## Coverage Tracking

| Item | Status | Test Coverage | Plan Doc | Notes |
| --- | --- | --- | --- | --- |
| Introduce coverage tooling | [ ] | `cargo tarpaulin` run (planned) | `docs/plans/test_utilities.md` | Evaluate tooling locally and document usage. |
| Automate in CI | [ ] | CI pipeline (planned) | `docs/plans/test_utilities.md` | Add coverage checks once tooling is validated. |
