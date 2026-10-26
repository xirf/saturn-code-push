# Security

## Reporting a vulnerability

Use the repository host's private vulnerability reporting feature when available. If it is not enabled, open an issue asking for a private reporting channel without including exploit instructions, credentials, or sensitive data. Do not attach database snapshots, signing keys, management tokens, session cookies, or capability URLs to a public issue.

Include the affected revision, deployment configuration, reproduction steps, and impact in the private report. This project does not currently publish a guaranteed response time or a long-term support policy.

## Deployment boundaries

Run the server behind HTTPS for public use. Set `PUBLIC_BASE_URL` to that exact public origin. Keep the container port bound to loopback or a private reverse-proxy network. Use the unprivileged, read-only container configuration in `server/compose.yaml` and preserve the data volume when rebuilding.

Keep management tokens, administrator password hashes, and signing private keys out of Git and browser storage. The Flutter client does not need administrator credentials. Browser management sessions use HTTP-only cookies and origin checks. Use separate data and credentials for development and tests.

Back up SQLite and object files as a coordinated snapshot before upgrading. Database migration can prevent older binaries from opening the data directory. Retain application release baselines and signing keys separately.

## Current limitations

- This is a single-administrator service without multiple roles or a durable audit log.
- Object download URLs are bearer capabilities without expiry. Treat them as secrets.
- Dart code-patch signing and asset ZIP integrity are separate. Asset metadata is not independently signed; SHA-256 does not protect against a compromised server replacing both metadata and bytes.
- Android has integration evidence; iOS and other platforms remain unverified.
- Network failure can cause the asset client to reuse a cached overlay. Offline devices cannot observe a server rollback immediately.
- Throughput, high availability, and production-scale failure recovery require deployment-specific validation.

The console's documented dependency audit limitation concerns Nuxt's development HTTPS tooling. Development HTTPS is disabled. The deployed image serves static console files from Rust and contains no Node runtime. See [dependency audit details](console/README.md#dependency-audit-limitation) before changing that tooling.

## Documentation and asset previews

Web documentation is generated only from repository-maintained Markdown with raw HTML disabled. Uploaded bundles are not documentation sources. Administrator-only asset inspection validates ownership and bounds archive/file/image work. It renders text as text and re-encodes supported image previews as PNG thumbnails.
