# ServeDocs Integration Plan

## Objective
Cover `ServeDocs::call()` behavior through integration-style tests validating markdown rendering, edit mode, and privilege filtering.

## Key Scenarios
- Requests without `Authorization` header should return the redirect page containing `pull_jwt_or_forward_to_login.js`.
- Requests with `?edit` should return an editable form prepopulated with the current markdown contents.
- Non-existent markdown paths should yield a `404 Not Found` response.
- JWT-based privilege differences should determine which sections render.

## Implementation Notes
- Bootstrap page now renders `redirectToLogin()` helper that uses `encodeURIComponent` and preserves the original docs path via query parameters.
- `pull_jwt_or_forward_to_login.js` fetches protected content with stored JWTs and clears invalid tokens when it receives `401`.
- Login experience consumes the `redirect` query parameter rather than `localStorage` state.
- Future doc tests should mock requests using async `Service::call` and decode the bootstrap script to assert helper presence.
