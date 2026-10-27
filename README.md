# Saturn Code Push

A self-hosted update server and web console for Flutter applications built with Shorebird. Saturn implements a tested subset of the Shorebird code-push protocol and adds an Android asset overlay workflow.

The server is a Rust modular monolith with PostgreSQL and filesystem object storage. The console uses Nuxt, Vue Composition API, Pinia Colada, Pinia, Tailwind, and shadcn-vue. Production runs as an unprivileged Linux container with Podman or Docker.

## Features

- Applications, immutable release baselines, patches, and delivery channels.
- Signed Dart-patch delivery using the application's own signing keys.
- Patch promotion, rollback, and restore.
- Separate asset ZIP publishing with image/text inspection in the console.
- Device-reported download, launch, and failure metrics.
- Accounts with Admin, Publisher, and Viewer roles, scoped CI API keys, and audit history.
- GitHub deployment reporting and commit comparisons across store destinations.
- Public, searchable web documentation at `/docs` on your instance.

Saturn is an independent implementation. It is not an official Shorebird self-hosted distribution. Building patches still uses Shorebird tooling. Android has device validation; iOS and other platforms are not verified. Read [compatibility and workflow differences](docs/usage.md) before adopting it.

## Quick start

From the repository root on a Linux host with Podman, Compose, OpenSSL, and Bash:

```bash
sh server/init-env.sh
podman compose -f server/compose.yaml up -d --build
```

Open `http://localhost:3000/setup`, enter the `SETUP_TOKEN` from the generated `server/.env`, and create your administrator account in the browser. The environment helper is optional: deployment dashboards can supply the database URL, public origin, and setup token directly. Before exposing an instance, configure its HTTPS origin and reverse proxy using the [deployment guide](server/README.md).

For Docker, use `docker compose` in place of `podman compose`. No host Rust or Node installation is required for the container build.

## Documentation

The same guides are published in the console's **Documentation** section and are available without signing in. Repository Markdown is the source; the web build generates searchable pages with navigation, heading links, and code-copy buttons.

- [Using Saturn with Shorebird](docs/usage.md)
- [Publish and inspect asset updates](docs/asset-support.md)
- [Flutter package integration](packages/saturn_assets/README.md)
- [Accounts, API keys, and PostgreSQL migration](docs/accounts-and-postgresql.md)
- [GitHub CI/CD](docs/github-deployments.md)
- [Server deployment and configuration](server/README.md)
- [Server architecture](server/ARCHITECTURE.md)
- [Console development](console/README.md)
- [Contributing](CONTRIBUTING.md)
- [Security](SECURITY.md)

## Development

Run Rust checks in Docker:

```bash
docker compose -f server/compose.dev.yaml run --rm --build server
```

Start the Nuxt development console:

```bash
docker compose -f console/compose.dev.yaml up
```

Run the isolated browser suite:

```bash
bash scripts/test-console.sh
```

See [Contributing](CONTRIBUTING.md) for formatting, linting, type checks, and test details. Development and test containers do not mount production data.

## License

[Apache License 2.0](LICENSE). Third-party libraries and fonts retain their own licenses; see [third-party notices](THIRD_PARTY_NOTICES.md).
