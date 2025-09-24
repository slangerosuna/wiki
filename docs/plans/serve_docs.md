# ServeDocs Integration Plan

## Objective
Cover `ServeDocs::call()` behavior through integration-style tests validating markdown rendering, edit mode, and privilege filtering.

## Key Scenarios
- Requests without `Authorization` header should return the redirect page containing `pull_jwt_or_forward_to_login.js`.
- Requests with `?edit` should return an editable form prepopulated with the current markdown contents.
- Non-existent markdown paths should yield a `404 Not Found` response.
- JWT-based privilege differences should determine which sections render.

## Implementation Notes
- Mock requests using `tower::ServiceExt::oneshot` against a constructed `ServeDocs` service.
- Populate temporary markdown files in a fixture directory for each test.
- Reuse JWT helpers from shared utilities to generate tokens for privilege scenarios.
