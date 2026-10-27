# Accounts, API keys, and PostgreSQL

Saturn stores the catalog, accounts, sessions, deployment reports and audit history in PostgreSQL 17. Artifact bytes remain in `DATA_DIR/objects`. SQLite is supported only as an import source, not as the live database.

## Start a fresh instance

Supply `DATABASE_URL`, `PUBLIC_BASE_URL`, and a random 32–256 character `SETUP_TOKEN` through your deployment platform's environment settings. Start Saturn and open its public URL. A fresh instance redirects to `/setup`; enter the setup token and choose your administrator username and password. You are signed in immediately.

No container shell or password-hashing command is needed. For local Compose, `sh server/init-env.sh` is an optional host-side helper that generates the environment file with random database/setup credentials. Run `podman compose -f server/compose.yaml up -d --build`, then use the token from that file in the browser. Hosting dashboards can set the same variables directly; Compose no longer requires a physical `.env` file when the variables are supplied by the platform.

The database URL and public origin remain deployment configuration: the server needs a database before onboarding can run. If `SETUP_TOKEN` is missing, the onboarding page explains how to configure it through deployment settings. The token is never returned by the status API or stored in browser storage. Setup requires the configured origin, applies login attempt/concurrency limits, and creates the account, session, and audit event in one database transaction. Concurrent requests can create only one first administrator.

Setup closes once **any account** exists, including disabled accounts. Removing or changing the setup token never reopens registration. Remove `SETUP_TOKEN` from deployment settings after onboarding. Later users, roles, keys, and password changes are managed in **Settings**.

`ADMIN_USERNAME`/`ADMIN_PASSWORD_HASH` and `set-login.sh` remain optional compatibility paths for automated first-account seeding. They seed only an empty account table and do not reset existing accounts. Public self-registration is not enabled.

## Roles

| Role | Workspace | Publishing | Accounts, keys, audit |
| --- | --- | --- | --- |
| Admin | Read | All management actions | Manage |
| Publisher | Read | Releases, patches, channels and deployment reports for existing apps | No |
| Viewer | Read | No | No |

Roles apply to the instance. API keys are separately scoped to one app. Every signed-in user can change their own password using their current password. Account changes invalidate the affected user's sessions and revoke their keys. Saturn prevents disabling or demoting the final active administrator.

Sessions last eight hours, use HTTP-only SameSite cookies, and are stored only as hashes. Cookie-authenticated mutations require the configured origin. Login has attempt and concurrent-password-check limits.

For local maintenance recovery, pass the new password on standard input, not as an argument:

```bash
read -r -s -p 'New password: ' SATURN_NEW_PASSWORD
printf '\n'
printf '%s' "$SATURN_NEW_PASSWORD" | docker compose -f server/compose.yaml run --rm -T code-push saturn-code-push --reset-password admin
unset SATURN_NEW_PASSWORD
```

This resets an existing account's password, revokes its sessions and keys, and records a local-maintenance audit event. It does not change roles or enable disabled accounts. Database credentials and host/container access are privileged administrative access.

## CI API keys

In **Settings → API keys**, choose a name, application, permission and expiry (1–365 days):

- **Report deployments** (`deployments:write`) can read and update deployment reports for its app.
- **Publish** (`publish`) can manage releases, patches, channels, artifacts and deployment reports for its app. It can list its own app and read the minimal self-hosted CLI organization/plan responses.

The full random token is returned once. Copy it into your CI provider's secret store. Saturn stores a SHA-256 hash and display prefix, creator, expiry, revocation state and last-used time. List responses never contain the token or hash. A key stops working when revoked, expired, or its creator loses access. Use the revocation button to rotate a key.

`ADMIN_TOKEN` is optional legacy CLI compatibility. It has broad application-management access and cannot identify an individual publisher. Its actions are explicitly attributed to **Legacy management token**. After migrating CLI jobs to named keys, remove it from the environment and restart Saturn. Access administration requires an administrator's browser session.

See [GitHub CI/CD](github-deployments.md) for reporting builds from Actions.

## Publishing attribution and destinations

Use destinations such as `app-store/testflight`, `app-store/production`, `google-play/internal`, `google-play/production`, `microsoft-store/production`, or a custom destination. Comparisons default to the previous recorded published deployment in the same repository and destination. Store review/publication state is supplied by you or CI; it is not fetched from Apple, Google or Microsoft.

Deployment history shows its original reporter and last updater. Audit history records the authenticated account or key, method, resource, app and time of successful management transactions. Changes and their audit records commit together. Request bodies, passwords and token values are not logged in the audit table. Imported deployments have unknown publishers because the old catalog did not record identities. Audit history is stored in PostgreSQL and is not a tamper-proof external ledger.

## Move an existing SQLite instance

Do not start old and new versions simultaneously. The import command reads the source database without modifying it, imports the application catalog in one PostgreSQL transaction, and requires empty target application tables. Accounts may already exist in the target. Browser sessions must be recreated through login after migration.

1. Stop the old server and take a coordinated backup of its SQLite database and entire objects directory. Keep its WAL/SHM files with it until a consistent backup is complete. Record the Compose project name and volume name.
2. Set `POSTGRES_PASSWORD` and `DATABASE_URL` in `server/.env`. Keep the existing `saturn-data` volume and Compose project name. URL-encode reserved characters in the database password, or generate a hexadecimal password. Keep the original file as a backup.
3. Build the new server image and start only PostgreSQL:

   ```bash
   docker compose -f server/compose.yaml build
   docker compose -f server/compose.yaml up -d postgres
   ```

4. Import the stopped catalog:

   ```bash
   docker compose -f server/compose.yaml run --rm code-push saturn-code-push --import-sqlite /data/catalog.sqlite3
   ```

5. Set `SETUP_TOKEN` in deployment settings if there are no accounts yet, start Saturn, and create the first administrator at `/setup`. Keep the original objects directory. Sign in, check apps/releases/deployments and downloads, and create named CI keys.

The importer accepts normalized SQLite schema versions 1–3. An original unversioned JSON catalog must first be upgraded using the previous SQLite server. Import preserves app/release/patch/artifact IDs, asset requirements, signatures, deployment IDs and the shared ID allocator. It resets PostgreSQL's deployment sequence after import. Artifact files are not copied or renamed; use the existing volume. The server refuses to start with an unimported `catalog.sqlite3` in `DATA_DIR`.

To roll back, stop the new server and restore the matching old SQLite-and-objects backup with the old image. New PostgreSQL changes are not mirrored back to SQLite. Never overwrite a live catalog with a backup from another version.

## Database operations

`DATABASE_URL` is required. The default Compose PostgreSQL service is on an internal container network and publishes no host database port. For an external database, use TLS with certificate verification (`sslmode=require` with this native-TLS connector), trusted CA certificates, and a least-privilege database user that can apply the required migrations. Do not log or commit the URL.

Numbered SQL migrations run transactionally at startup under a schema-scoped PostgreSQL advisory lock. Unknown newer versions are rejected. Repositories use parameterized PostgreSQL SQL and a bounded connection pool on Tokio's blocking workers. Use cases currently serialize under the same advisory lock to preserve publication and administrative invariants; this is not a high-throughput, multi-region design. Multiple server processes must share the same objects filesystem.

Back up PostgreSQL and artifact files together while application writes are stopped. Preserve signing keys separately. `compose down -v` deletes the database and artifact volumes. Test restore procedures before depending on backups.

The Toasty ORM was evaluated at version 0.11. Its roadmap lists database constraints and transaction controls Saturn needs, including composite foreign keys and partial unique constraints. This migration retains explicit SQL repositories and migrations; it does not adopt Toasty yet.
