# Artifact storage

Saturn stores binary releases, binary patches, and asset bundles on local disk by default. Set `STORAGE_BACKEND=s3` to put new uploads in a private S3 compatible bucket. AWS S3, Cloudflare R2, and MinIO use the same integration. No client or CI publishing changes are required.

## Folder organization

New uploads use immutable database IDs rather than mutable display names or user filenames:

```text
apps/{app-id}/releases/{release-id}/binaries/{artifact-id}
apps/{app-id}/releases/{release-id}/patches/{patch-id}/binaries/{artifact-id}
apps/{app-id}/releases/{release-id}/patches/{patch-id}/assets/{bundle-id}
```

Special characters in app IDs are encoded. S3 keys also have a configurable prefix (default `saturn`) and a random upload-attempt suffix. This keeps concurrent attempts independent. Platform and architecture metadata remain in PostgreSQL, linked by artifact ID.

Existing uploads keep their original `DATA_DIR/objects/{id}` or `asset-{id}` layout. PostgreSQL records the location of every new upload atomically with publication. Changing the backend affects new uploads only. It does not move or delete existing files.

## Configure S3

Create a private bucket, then set these variables in your hosting provider's environment or secret settings and restart Saturn:

```dotenv
STORAGE_BACKEND=s3
STORAGE_S3_BUCKET=saturn-artifacts
STORAGE_S3_REGION=us-east-1
STORAGE_S3_PREFIX=saturn
AWS_ACCESS_KEY_ID=your-access-key
AWS_SECRET_ACCESS_KEY=your-secret-key
```

Use temporary credentials with `AWS_SESSION_TOKEN` if needed. AWS workload credentials are also supported: EC2 instance roles, ECS task roles, and web identity (`AWS_WEB_IDENTITY_TOKEN_FILE`, `AWS_ROLE_ARN`, optional `AWS_ROLE_SESSION_NAME`). Mount credential files through your hosting platform when using web identity. The Compose example passes static/temporary credential variables; add the workload role variables or mounts for your deployment as appropriate. Empty static credential variables are ignored.

For AWS S3 the default endpoint is `https://s3.{region}.amazonaws.com`. For another provider, set an endpoint origin without a bucket, credentials, path, query, or fragment:

```dotenv
# Cloudflare R2
STORAGE_S3_ENDPOINT=https://ACCOUNT_ID.r2.cloudflarestorage.com
STORAGE_S3_REGION=auto
```

Path-style requests are used. For local MinIO, set an endpoint such as `http://localhost:9000`, its configured region (usually `us-east-1`), and explicitly set `STORAGE_S3_ALLOW_HTTP=true`. Use HTTPS in production. The endpoint must be reachable by both the server and downloading devices; an internal Docker hostname will not work for device downloads.

Allow outbound access to the configured endpoint and any credential-provider endpoints. Grant the server `s3:GetObject`, `s3:PutObject`, `s3:DeleteObject`, and `s3:AbortMultipartUpload` on the selected bucket prefix. Delete is used to clean up rejected publication attempts. Bucket listing and public access are not required. If the bucket uses a customer-managed KMS key, also grant the required KMS permissions.

Administrators can see the active backend, bucket, endpoint, region, and prefix under **Settings → Artifact storage**. Credentials are never returned to the dashboard. Configuration is controlled through hosting settings, rather than storing cloud credentials in the application database.

## Uploads, downloads, and previews

Saturn streams incoming uploads to temporary local disk while checking their registered size and SHA-256. Binary diffs retain their existing hash semantics: the registered hash describes the reconstructed executable. Validated uploads are transferred to S3 in bounded multipart chunks, then marked uploaded in PostgreSQL. Failed or concurrent rejected publications do not replace successful bytes. A storage failure leaves the registration pending so the upload can be retried.

Public capability download URLs stay on the Saturn origin. After validating the capability, S3 downloads redirect to a signed GET URL valid for five minutes. Clients must follow HTTP 307 redirects. The private bucket remains private; signed URLs are bearer capabilities and should not be logged or shared. Local downloads continue supporting byte ranges. S3 handles byte ranges after the redirect.

Authenticated asset inspection and ZIP downloads fetch a temporary local copy, keep the existing ownership checks, and remove that copy afterward. Allow enough temporary disk for concurrent incoming uploads and asset inspections. S3 removes the need for permanent artifact disk storage; it does not eliminate temporary disk use.

## Switching and backups

Keep the data volume mounted while any older local artifacts exist. Switching back to `local` preserves access to S3 files if the original S3 configuration and credentials remain available. Changing the prefix is safe: existing keys are stored in PostgreSQL. Changing the bucket, region, or endpoint while existing S3 artifacts remain there causes an explicit error for those files, rather than silently reading a different bucket. Multiple historical bucket configurations are not supported in this version.

Automatic bulk migration of older files is not included. Back up PostgreSQL and all storage locations together. Do not manually reorganize recorded files or expire published objects with bucket lifecycle rules. Configure a lifecycle rule to abort incomplete multipart uploads. A process crash, uncertain network response, or failed cleanup can leave an unreferenced completed object; retain it for manual reconciliation against `artifact_objects`, rather than deleting it based only on age. An uncertain database commit deliberately preserves bytes to avoid breaking a potentially successful publication.

## Integration tests

The server's `s3_publication_download_inspection_and_rollback` test is ignored in ordinary runs because it needs an external, disposable S3 bucket. Point `TEST_DATABASE_URL` at a disposable PostgreSQL database, set the storage variables and credentials above for a test bucket, and run:

```sh
cargo test --manifest-path server/Cargo.toml s3_publication -- --ignored
```

Test credentials additionally need `s3:ListBucket` so the test can check cleanup. The test writes objects and intentionally tries a missing bucket; do not point it at production storage. It can run against an S3 emulator such as Moto or a local MinIO instance.
