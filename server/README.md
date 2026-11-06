# Saturn Code Push

An independent Rust/Axum service implementing a tested subset of Shorebird's public code-push protocol. It contains the management/device API, an original dashboard, and basic patch-event analytics.

The API, web console, and public documentation share the same origin. Configure your own HTTPS origin for deployment.

## Start

Deployment targets Linux with Podman. Development and validation run in Docker containers; no host Rust installation is required. Run these commands from the repository root:

```sh
sh server/init-env.sh
# Edit server/.env: set PUBLIC_BASE_URL to the HTTPS origin used by your proxy.
podman compose -f server/compose.yaml up -d --build
```

Open the configured origin in your browser. A fresh instance redirects to `/setup`, where you enter the one-time `SETUP_TOKEN` and create the first administrator. There is no password-hashing command or container shell step. Compose starts PostgreSQL and waits for it to become healthy.

`init-env.sh` is an optional host-side convenience that creates an owner-readable `server/.env` with random database and setup credentials. Hosting platforms can supply `DATABASE_URL`, `PUBLIC_BASE_URL`, and a random 32–256 character `SETUP_TOKEN` through their environment settings instead. Database connectivity must be configured before the application can start. First-run setup stays disabled if no setup token is configured.

Accounts are stored in PostgreSQL. Setup closes once any account exists; remove `SETUP_TOKEN` from deployment settings afterward. Optional `ADMIN_USERNAME`/`ADMIN_PASSWORD_HASH` and `set-login.sh` remain compatible with automated seeding of an empty account table. Use **Server settings** for accounts, Admin/Publisher/Viewer roles, password changes, and audit history, and each application's **Settings** for its app-scoped API keys and GitHub connection. Sessions last eight hours; password resets and access changes revoke sessions and keys. `ADMIN_TOKEN` is optional legacy compatibility; prefer named keys for CLI/CI publishing.

For subsequent rebuilds, use `podman compose -f server/compose.yaml up -d --build`.

Docker exposes `127.0.0.1:3000`; PostgreSQL has no published host port. Configure an HTTPS reverse proxy with `PUBLIC_BASE_URL` matching the browser origin. `DATABASE_URL` is required. The unprivileged server has a read-only image filesystem and uses `saturn-data` for objects. PostgreSQL uses a separate `saturn-postgres` volume. Keep the Compose project name stable; `compose down -v` deletes both volumes.

**Existing SQLite installations require an explicit import.** Follow [Accounts, API keys, and PostgreSQL](../docs/accounts-and-postgresql.md) before starting the new server. Preserve your database-and-objects backup. The importer retains IDs and object paths; old browser sessions expire.

For application developers, [Using Saturn with Shorebird](../docs/usage.md) covers CLI setup, signed baselines, patches, and the differences from hosted Shorebird. [Asset updates](../docs/asset-support.md) documents the separate ZIP publishing and dashboard inspection workflow.

## Features

- App creation, release registration, per-platform release activation, channels and patch registration.
- Metadata-first artifact registration followed by streamed, one-use multipart upload URLs.
- Release SHA-256/size checking, immutable uploads, range-capable artifact downloads.
- Patch checks scoped by app, exact release version, platform, architecture and channel.
- Promotion, server-driven rollback, and rollforward.
- Customer-owned patch signing: the CLI signs the reconstructed-artifact hash; the enabled engine verifies it on the device. Use `--public-key-path` for release and `--private-key-path` for patch. Managed signing is not implemented.
- Nuxt/Vue dashboard with shadcn-vue components, application/release navigation, patch promotion/rollback dialogs, responsive layouts, and ECharts analytics. Pinia Colada owns API state and mutations; Pinia owns UI preferences.
- Download, first-launch, failure and unique-reporting-install counts, plus daily UTC event totals. These cover the retained client-report window, not total active devices or adoption. Events with an optional `event_id` are deduplicated per application while retained; legacy events without IDs count again. `__patch_first_launch__` and `__patch_health__` are separate from download and install reports. Reuse the same event ID for retries; never reuse it for a different event. Pruning ends the deduplication window. Reports remain untrusted device claims.
- A 64 KiB device request body cap, a 100-request/second process-wide device API limit, and bounded telemetry retention (10,000 recent events).

Artifacts are capped at 256 MiB. Download URLs are unguessable bearer capabilities without expiry; treat release URLs as secrets. Patch uploads are checked for size; verification of the reconstructed artifact and its signature occurs on the device, not at upload time.

## Validation

```sh
docker compose -f server/compose.dev.yaml run --rm --build server
docker compose -f server/compose.dev.yaml run --rm server cargo fmt --all -- --check
docker compose -f server/compose.dev.yaml run --rm server cargo clippy --locked --all-targets -- -D warnings
```

The development image includes Rustfmt and Clippy. Cargo registry and build caches use separate named volumes. This environment does not mount the production data volume. Replace `docker` with `podman` to run the same checks on the deployment engine.

See [architecture](ARCHITECTURE.md) for module ownership, entity files, transaction boundaries, and the Nuxt console structure.

## Production hostname

Build the image from the repository root using `server/Dockerfile`; it includes the current console build. Generate a separate production management token. Set `PUBLIC_BASE_URL=https://updates.example.com` and configure that hostname's reverse proxy/tunnel to the server port. `PUBLIC_BASE_URL` must be an origin without a path, credentials, query, or fragment. Build new client releases with `base_url: https://updates.example.com` in their bundled `shorebird.yaml`; changing a server variable does not rewrite an already-installed application's endpoint.

Back up PostgreSQL and object files together while application writes are stopped. Keep the exact release baselines: patch reconstruction depends on them. Signing private keys live separately and must also be backed up.

Saturn implements a subset of the Shorebird protocol. OAuth, billing, managed signing, background artifact analysis, and iOS validation remain unimplemented. PostgreSQL repositories use parameterized SQL, a bounded pool, and transaction locks on Tokio's blocking workers. Use cases still serialize to preserve publishing invariants, and some queries filter entity lists in memory. Expiring storage URLs, monitoring, operational restore testing, and load testing remain deployment responsibilities. Building patches relies on Shorebird-distributed compiler/engine artifacts.

## Frontend development

The `console/` Nuxt application generates static files in `.output/public`, served by Rust. Docker builds both layers in one image. The production image has no Node runtime. Fonts are bundled locally; no runtime font CDN is needed.

See [the console guide](../console/README.md) for feature folders, generated shadcn styles, session-cache handling, the dependency-audit limitation, and Linux browser tests.

```sh
docker compose -f console/compose.dev.yaml up
bash scripts/test-console.sh
```

The development console uses port 3001 and authenticated server data. Its development API must use a `PUBLIC_BASE_URL` matching the browser origin; the proxy preserves origin checks.
