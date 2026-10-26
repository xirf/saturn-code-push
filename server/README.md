# Saturn Code Push

An independent Rust/Axum service implementing a tested subset of Shorebird's public code-push protocol. It contains the management/device API, an original dashboard, and basic patch-event analytics. It does not contain or serve the Quick GMJ customer website.

Development hostname: **https://saturn.andka.id**. Planned production hostname: **https://apps.gmjroaming.com**. Both the API and dashboard use the same origin. Quick GMJ remains a separate application at `quick.gmjroaming.com`.

## Start

From this repository root, with Docker Desktop running:

```powershell
.\server\start.ps1
```

The helper creates an ignored `server/.env` with a cryptographically random API token on first use. Configure the dashboard's username and password with:

```powershell
.\server\set-login.ps1 -Username admin
```

The script prompts for a password and confirmation, stores only an Argon2id hash in `server/.env`, and restarts the service. Passwords must contain at least 12 characters. Password resets invalidate existing browser sessions. The dashboard uses eight-hour, server-side sessions with HTTP-only, SameSite=Strict cookies (Secure on HTTPS), origin checks on cookie-authenticated mutations, and sign-in rate limiting. Session tokens are hashed in SQLite and survive container restarts.

`ADMIN_TOKEN` remains for CLI/API use; the browser no longer asks for or stores that token. Keep `.env` and signing keys out of Git. This remains a single-administrator service; it does not provide user registration or multiple roles.

Docker exposes `127.0.0.1:3000`. The existing Cloudflare Tunnel provides HTTPS for `saturn.andka.id`. The container itself speaks HTTP behind the tunnel; no Android certificate installation is needed. `/health` is public; management APIs and analytics require `Authorization: Bearer <ADMIN_TOKEN>`.

The service runs as an unprivileged container user. The `server_saturn-data` Docker volume holds the SQLite catalog and artifact files. Rebuilds preserve it. Do not run `docker compose down -v` unless you intend to erase releases and patches.

## Features

- App creation, release registration, per-platform release activation, channels and patch registration.
- Metadata-first artifact registration followed by streamed, one-use multipart upload URLs.
- Release SHA-256/size checking, immutable uploads, range-capable artifact downloads.
- Patch checks scoped by app, exact release version, platform, architecture and channel.
- Promotion, server-driven rollback, and rollforward.
- Customer-owned patch signing: the CLI signs the reconstructed-artifact hash; the enabled engine verifies it on the device. Use `--public-key-path` for release and `--private-key-path` for patch. Managed signing is not implemented.
- React/Vite dashboard with Radix Themes components, application/release navigation, patch promotion/rollback dialogs, responsive layouts and Recharts analytics.
- Download, first-launch, failure and unique-reporting-install counts, plus daily UTC event totals. These cover the retained client-report window, not total active devices or adoption. Duplicate events count again.
- A 64 KiB device request body cap, a 100-request/second process-wide device API limit, and bounded telemetry retention (10,000 recent events).

Artifacts are capped at 256 MiB. Download URLs are unguessable bearer capabilities without expiry; treat release URLs as secrets. Patch uploads are checked for size; verification of the reconstructed artifact and its signature occurs on the device, not at upload time.

## Validation

```powershell
cargo test --manifest-path server/Cargo.toml
cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings
```

`scripts/saturn-contract.dart` exercises the **actual Shorebird Dart client** against the running server, including its response decoders and the two-step artifact uploads. It creates a labelled protocol-fixture app whose bytes are not an executable patch. Run it using the downloaded Shorebird SDK and workspace package configuration:

```powershell
$sb = (Get-Content .tooling/shorebird-path.txt -Raw).Trim()
$rev = (Get-Content "$sb/bin/internal/flutter.version" -Raw).Trim()
& "$sb/bin/cache/flutter/$rev/bin/cache/dart-sdk/bin/dart.exe" "--packages=$sb/.dart_tool/package_config.json" scripts/saturn-contract.dart
```

See [the isolated probe](../examples/saturn_probe/README.md) for the real Android test. Tooling used for this pass: Shorebird CLI `1.6.133`, source commit `2bb03a04ad157f014f16fe588f97317de7ad9b02`; Flutter `3.47.8`, revision `baf23dc375905d9103ca4325b062cadb7b8f90e5`; engine `5a3c299daf4061281fe2d8909b690e29cfb4e9c3`.

## Production hostname

Deploy this `server/` directory independently of Quick GMJ. Generate a separate production management token. Set `PUBLIC_BASE_URL=https://apps.gmjroaming.com` and configure that hostname's reverse proxy/tunnel to the server port. Build new client releases with `base_url: https://apps.gmjroaming.com` in their bundled `shorebird.yaml`; changing a server variable does not rewrite an already-installed application's endpoint.

Back up the catalog and object files together while the container is stopped, or use SQLite's backup facility with a coordinated object snapshot. Keep the exact release baselines: patch reconstruction depends on them. Signing private keys live separately and must also be backed up.

This is a single-operator prototype, not a complete Shorebird service. Multi-user roles, OAuth, billing, managed signing, percentage rollouts, durable audit history, background artifact analysis, and iOS validation are not implemented. The catalog is serialized under one process lock into a SQLite snapshot; it is intended for low-volume evaluation. A production implementation should use normalized tables and event ingestion, expiring storage URLs, monitoring, backups and load testing. Building patches still relies on Shorebird-distributed compiler/engine artifacts.


## Frontend development

The `console/` React application builds to static assets served by Rust. Docker builds both layers in one image. Fonts are bundled locally; no runtime font CDN is needed.

```powershell
cd console
npm ci
npm run build
npm run dev
```

`http://127.0.0.1:5173/?preview=1` is a development-only design preview with explicitly labelled illustrative data. Production always uses authenticated server data. Sign-in origin checks are bound to `PUBLIC_BASE_URL`; test real login at that hostname, not the Vite preview origin.
