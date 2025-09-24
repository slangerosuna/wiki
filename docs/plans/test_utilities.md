# Test Utilities Plan

## Objective
Create reusable fixtures and helper functions to streamline integration tests across the project.

## Tasks
- Provide a `TempDb` helper that encapsulates SQLite setup and teardown.
- Supply JWT generation helpers parameterized by privilege level.
- Expose filesystem fixtures for documentation tests.

## Notes
- Place shared utilities under `tests/support/` or `tests/util.rs` for consumption by all integration suites.
