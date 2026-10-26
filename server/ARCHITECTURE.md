# Server architecture

Saturn is a modular monolith: one Rust crate, one Linux container, and one SQLite database. Business modules own their entities, repositories, use cases, and HTTP adapters. Module declarations use `name.rs` beside `name/`; there are no `mod.rs` files.

## Layout

```text
src/
├── main.rs                  # CLI entry point, listener, SIGINT/SIGTERM shutdown
├── lib.rs                   # Application construction API
├── bootstrap.rs             # Shared infrastructure and identity configuration
├── config.rs                # Environment validation
├── migration.rs             # Versioned, atomic legacy import
├── http.rs                  # Router composition and shared response policies
├── http/
│   ├── admin.rs             # Read-only composition of module snapshots
│   ├── compatibility.rs     # Fixed single-operator CLI responses
│   ├── console.rs           # Current console entry point
│   └── middleware.rs        # Auth, origin checks, throttling, protocol adaptation
├── platform.rs
├── platform/
│   ├── database.rs          # Transaction runner and blocking-pool boundary
│   ├── error.rs             # Safe HTTP error responses
│   ├── objects.rs           # Internal object paths
│   └── values.rs            # IDs, timestamps, bounded string extraction
├── modules.rs
├── modules/
│   ├── applications.rs
│   ├── applications/
│   ├── distribution.rs
│   ├── distribution/
│   ├── artifacts.rs
│   ├── artifacts/
│   ├── identity.rs
│   ├── identity/
│   ├── telemetry.rs
│   └── telemetry/
├── tests.rs
└── tests/                   # Feature, migration, and security regressions
```

Each entity has its own model and repository file. For example:

```text
modules/applications/
├── api.rs
├── http.rs
├── service.rs
├── models.rs
├── models/
│   ├── app.rs
│   └── channel.rs
├── repositories.rs
└── repositories/
    ├── app_repository.rs
    └── channel_repository.rs
```

`distribution` uses separate `release.rs`, `patch.rs`, `release_repository.rs`, and `patch_repository.rs` files. `artifacts` separates binaries and asset bundles. Identity separates credential configuration from persisted sessions. Telemetry has an event model and event repository. Wire request types live in `dto.rs` where they are shared by adapters and services.

## Ownership

| Module       | Owns                                                                              | Tables                                          |
| ------------ | --------------------------------------------------------------------------------- | ----------------------------------------------- |
| Applications | Apps and channels                                                                 | `applications_apps`, `applications_channels`    |
| Distribution | Releases, patch numbering, promotion, rollback, device patch selection            | `distribution_releases`, `distribution_patches` |
| Artifacts    | Binary and asset-bundle registration, transfer, capabilities, upload immutability | `artifacts_binaries`, `artifacts_bundles`       |
| Identity     | Password verification, session rotation, expiry and revocation                    | `dashboard_sessions`                            |
| Telemetry    | Event validation, bounded retention and analytics                                 | `telemetry_events`                              |

Infrastructure owns the database schema version and the shared ID sequence, which preserves protocol IDs during migration. The legacy snapshot is retained for diagnosis and is never used for current reads or writes.

## Module rules

- The application router only assembles modules and shared response policies. Each module owns its paths through `http::routes`; resource submodules own relative paths. Distribution nests release details, release patches, and artifact routes under `/api/v1/apps/{app}/releases/{release}`. Shared management/device policy helpers keep authorization and throttling explicit.
- Call another module through `api.rs`. Its models and repositories remain private modules; Rust visibility restricts repository operations to their owner.
- Exchange owned query results across modules. Callers cannot mutate another module's persisted state through returned models.
- Keep HTTP extractors, multipart parsing, response headers, and cookies in HTTP adapters. Services validate business rules; repositories execute parameterized SQL.
- Give every new entity its own model file and repository file. Keep repository SQL and row decoding together.
- Keep release and patch rules in distribution. Promotion needs application/channel queries and artifact readiness queries. Artifact registration queries distribution to validate ownership. These read dependencies intentionally share a transaction; they do not write one another's tables.
- Schema initialization and migration imports are explicit infrastructure entry points on module APIs. Request handlers must not use migration import functions.
- Add shared infrastructure only for a concrete common need. Business rules do not belong in `platform`.

## Transactions and files

The HTTP adapter invokes a complete use case through `Database::call`. Tokio runs it on the blocking pool. The database runner starts an immediate SQLite transaction, runs the use case, and commits only on success. No database lock crosses an `await`. This preserves atomic app/default-channel creation, ID allocation, registration, and state changes.

Every entity has its own table and repository. Foreign keys enforce app/release/patch relationships, including composite ownership constraints for artifact records. Unique indexes prevent duplicate artifact targets. Dynamic platform/signing metadata and patch channel lists remain JSON columns; there is no global serialized catalog in the active data path.

Uploads stream into owner-readable temporary files using `tempfile`. Validation checks registered size and the applicable SHA-256 hash. Publication rechecks the one-use state within the database transaction, atomically publishes the file, syncs it, and commits. The blocking task completes publication or cleanup if the HTTP request disconnects. Failed commits remove published files. A retry can replace an orphan from a crash before commit; already-published records reject replacement. Device verification still owns reconstructed patch signature verification.

Database transactions are serialized through one connection. Several repositories currently load entity lists for filtering. These are explicit throughput limits, not a distributed or horizontally scalable storage design. Introduce indexed repository queries when measured use requires them; keep the module APIs stable while doing so.

## Migration and operations

Schema version 1 imports the original catalog in one transaction. Malformed data or constraint failures roll back the complete migration. Sessions and object filenames remain unchanged. The original catalog becomes `legacy_catalog_v0`; a compatibility view makes the original binary fail catalog deserialization at startup instead of writing conflicting IDs into the migrated store. Newer unknown schema versions are rejected.

Back up SQLite and object files together before upgrading. An application downgrade requires restoring the matching pre-migration backup. Do not run old and new server versions against the same data volume.

The production container uses UID/GID 10001, a read-only image filesystem, a writable data volume, no extra Linux capabilities, and graceful termination. TLS remains at the reverse proxy. Bearer tokens are compared as fixed-length SHA-256 digests using `subtle`; browser sessions retain Argon2id login, hashed tokens, Secure/HttpOnly/SameSite cookies, and origin checks. Management bodies are capped at 64 KiB; device JSON at 64 KiB; login JSON at 4 KiB. File limits are enforced during streaming.

## Console

The console uses Nuxt and TypeScript with Vue Composition API (`<script setup>`), Pinia, Pinia Colada, Tailwind, and generated shadcn-vue components. Zod validates requests and responses; ECharts renders telemetry. See [the console guide](../console/README.md) for details and dependency-audit limitations.

```text
console/
├── nuxt.config.ts
├── app/
│   ├── app.vue
│   ├── pages/               # Route composition only
│   ├── layouts/             # Authenticated shell and sign-in layout
│   ├── middleware/          # Navigation guards; server still enforces auth
│   ├── components/ui/       # Shared accessible UI primitives
│   ├── features/
│   │   ├── applications/
│   │   ├── distribution/
│   │   ├── artifacts/
│   │   ├── identity/
│   │   └── telemetry/
│   ├── lib/api/             # Same-origin typed HTTP transport
│   ├── stores/              # Pinia client-only preferences/UI state
│   └── assets/css/          # Tailwind entry and design tokens
└── tests/
```

Within each feature, separate `components/`, `composables/`, `queries/`, and `types/`. Pinia Colada owns remote query state, mutation state, invalidation, and cache lifetime. Pinia owns client state such as navigation and display preferences; do not duplicate API responses in stores. Components consume feature composables instead of calling raw HTTP throughout the UI.

Keep same-origin HTTP-only session cookies and backend origin checks. Clear authenticated query caches on logout and authentication changes. Never put the management bearer token or session token in browser storage. Nuxt generates a static SPA served by Rust in the same container. The build externalizes executable bootstrap scripts and disables runtime code generation to preserve the server CSP. No Node runtime is deployed.
