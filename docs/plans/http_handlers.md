# HTTP Handler Test Plan

## Objective
Ensure `/api/register` and `/api/login` operate correctly through end-to-end Axum requests under controlled database conditions.

## Planned Scenarios
- Successful registration returns a JWT and privilege payload.
- Duplicate username registration responds with `500` and leaves existing user intact.
- Login with valid credentials returns `200` and previously issued privileges.
- Login with invalid credentials returns `401` and no token.
- Database state is verified after registration and privilege updates.

## Implementation Notes
- Leverage a temporary SQLite database per test.
- Construct the router via `wiki::` exports for reuse in integration tests.
- Use `tower::ServiceExt::oneshot` to simulate HTTP requests.
