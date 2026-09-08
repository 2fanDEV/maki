# Repository conventions

## Rust service and API modules

Organize each service using this structure:

```text
<service>/
├── mod.rs
├── api/
│   ├── mod.rs
│   ├── request.rs
│   └── response.rs
└── tests.rs
```

- Keep service state, business logic, orchestration, and calls to domain code in the service's `mod.rs`.
- Put `impl Service for <name>Service`, route registration, HTTP extraction, status mapping, SSE formatting, and Aide documentation in the service's `api/mod.rs`. HTTP handlers call the parent service's logic.
- Put API request bodies, path/query parameters, and input settings in `api/request.rs`.
- Put API response types, API errors, and response conversions in `api/response.rs`.
- Keep API-specific structs and enums in these request/response modules. Derive their schemas there. Reuse shared domain types from their existing modules instead of duplicating or moving them into the API layer.
- Keep request/response modules for placeholder endpoints without inventing payload types.
- Keep application router composition and `/openapi.json` setup in `src/api/mod.rs`. Keep `main.rs` limited to application startup.
- Keep routing and documentation concise. Do not introduce a second set of endpoint registrations for OpenAPI generation.

## Verification

- The API layer does not need tests. Do not add API-layer or OpenAPI schema tests.
- Preserve existing service behavior and algorithm tests outside API modules. Adjust imports and call sites when moving code.
- Run existing tests, Clippy, and formatting checks for Rust changes. Do not modify unrelated files merely to clear existing warnings or formatting issues.
