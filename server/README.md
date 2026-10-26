# Saturn Code Push

An independent Rust/Axum service implementing a tested subset of Shorebird's public code-push protocol. It contains the management/device API, an original dashboard, and basic patch-event analytics.

The API, web console, and public documentation share the same origin. Configure your own HTTPS origin for deployment.

## Start

Deployment targets Linux with Podman. Development and validation run in Docker containers; no host Rust installation is required. Run these commands from the repository root:

```sh
sh server/init-env.sh
# Edit server/.env: set PUBLIC_BASE_URL to the HTTPS origin used by your proxy.
podman compose -f server/compose.yaml build
bash server/set-login.sh admin
```

The environment helper creates an ignored, owner-readable `server/.env` with a random management token. The login helper prompts for a password and confirmation, hashes it inside the container, updates the environment file atomically, and starts the service. Use `CONTAINER_ENGINE=docker bash server/set-login.sh admin` when running with Docker.

For subsequent rebuilds:

```sh
podman compose -f server/compose.yaml up -d --build
```

The script prompts for a password and confirmation, stores only an Argon2id hash in `server/.env`, and restarts the service. Passwords must contain at least 12 characters. Password resets invalidate existing browser sessions. The dashboard uses eight-hour, server-side sessions with HTTP-only, SameSite=Strict cookies (Secure on HTTPS), origin checks on cookie-authenticated mutations, and sign-in rate limiting. Session tokens are hashed in SQLite and survive container restarts.

`ADMIN_TOKEN` remains for CLI/API use; the browser no longer asks for or stores that token. Keep `.env` and signing keys out of Git. This remains a single-administrator service; it does not provide user registration or multiple roles.

Docker exposes `127.0.0.1:3000`. Configure your own HTTPS reverse proxy or tunnel. The container speaks HTTP on the private side of that proxy. `/health` is public; management APIs and analytics require `Authorization: Bearer <ADMIN_TOKEN>`.

The service runs as an unprivileged container user with a read-only image filesystem, no Linux capabilities, and a writable data volume. The Compose volume `saturn-data` holds SQLite and artifact files. Its engine-managed name depends on the Compose project name; the existing default is `server_saturn-data`. Keep the project name stable when upgrading. Rebuilds preserve the volume. `compose down -v` deletes it.

SQLite upgrades automatically on startup in one transaction. The migration imports the original catalog into module-owned tables and preserves IDs, object filenames, and browser sessions. It retains the original snapshot as `legacy_catalog_v0` and installs a compatibility guard that prevents the old binary from starting against the migrated database. Back up the database and objects together before upgrading; rolling back the image also requires restoring that backup.

For application developers, [Using Saturn with Shorebird](../docs/usage.md) covers CLI setup, signed baselines, patches, and the differences from hosted Shorebird. [Asset updates](../docs/asset-support.md) documents the separate ZIP publishing and dashboard inspection workflow.

## Features

- App creation, release registration, per-platform release activation, channels and patch registration.
- Metadata-first artifact registration followed by streamed, one-use multipart upload URLs.
- Release SHA-256/size checking, immutable uploads, range-capable artifact downloads.
- Patch checks scoped by app, exact release version, platform, architecture and channel.
- Promotion, server-driven rollback, and rollforward.
- Customer-owned patch signing: the CLI signs the reconstructed-artifact hash; the enabled engine verifies it on the device. Use `--public-key-path` for release and `--private-key-path` for patch. Managed signing is not implemented.
- Nuxt/Vue dashboard with shadcn-vue components, application/release navigation, patch promotion/rollback dialogs, responsive layouts, and ECharts analytics. Pinia Colada owns API state and mutations; Pinia owns UI preferences.
- Download, first-launch, failure and unique-reporting-install counts, plus daily UTC event totals. These cover the retained client-report window, not total active devices or adoption. Duplicate events count again.
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

Back up the catalog and object files together while the container is stopped, or use SQLite's backup facility with a coordinated object snapshot. Keep the exact release baselines: patch reconstruction depends on them. Signing private keys live separately and must also be backed up.

This is a single-operator service, not a complete Shorebird implementation. Multi-user roles, OAuth, billing, managed signing, percentage rollouts, durable audit history, background artifact analysis, and iOS validation are not implemented. Each entity has an owned repository and relational table. SQLite transactions use one connection on Tokio's blocking pool; throughput remains limited by serialized database access and some queries still filter entity lists in memory. Expiring storage URLs, monitoring, operational backups, and load testing remain future work. Building patches still relies on Shorebird-distributed compiler/engine artifacts.

## Frontend development

The `console/` Nuxt application generates static files in `.output/public`, served by Rust. Docker builds both layers in one image. The production image has no Node runtime. Fonts are bundled locally; no runtime font CDN is needed.

See [the console guide](../console/README.md) for feature folders, generated shadcn styles, session-cache handling, the dependency-audit limitation, and Linux browser tests.

```sh
docker compose -f console/compose.dev.yaml up
bash scripts/test-console.sh
```

The development console uses port 3001 and authenticated server data. Its development API must use a `PUBLIC_BASE_URL` matching the browser origin; the proxy preserves origin checks.
