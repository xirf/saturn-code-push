# Contributing to Saturn

Small, focused contributions are welcome. Describe the behavior being changed and how it was validated. Use issues for reproducible bugs and feature proposals. Keep credentials, app binaries, database snapshots, signing keys, and customer data out of issues and pull requests.

## Development environment

Use Docker for development and validation. Production targets Linux with Podman or Docker. Install a container engine with Compose support; the test runner also requires Bash and OpenSSL. Flutter integration work needs a compatible Shorebird/Flutter toolchain separately from the server container.

Follow [server setup](server/README.md), [console development](console/README.md), and [architecture](server/ARCHITECTURE.md). Keep local data and credentials separate from deployed instances.

## Project structure

- `server/src/modules/`: application, distribution, artifact, identity, and telemetry modules.
- `console/app/features/`: UI components, composables, queries, and types by feature.
- `packages/saturn_assets/`: Flutter asset overlay package.
- `docs/`: canonical user guides, also rendered into web documentation.
- `scripts/publish-asset-bundle.py`: supported Android bundle publisher.
- `scripts/test-console.sh`: isolated Linux browser test runner.

Keep entity models and repositories separate. Use modern Rust module files alongside their directories. Let each HTTP module own its relative routes. Keep business rules in services and database access in repositories.

For the console, use Composition API and the existing shadcn-vue primitives. Pinia Colada owns server state; Pinia holds UI preferences. Use libraries where they simplify maintenance. Keep code readable and formatted; do not compress handlers or templates into dense one-line expressions.

## Validation

Run the server tests, formatting check, and Clippy:

```bash
docker compose -f server/compose.dev.yaml run --rm --build server

docker compose -f server/compose.dev.yaml run --rm \
  server cargo fmt --all -- --check

docker compose -f server/compose.dev.yaml run --rm \
  server cargo clippy --locked --all-targets -- -D warnings
```

Check the console and format it after edits:

```bash
docker compose -f console/compose.dev.yaml run --rm \
  console sh -c 'npm ci && npm run format && npm run check'
```

Run `bash scripts/test-console.sh` for browser changes. It builds an isolated server with temporary credentials and disposable data, then checks navigation, authentication, patch actions, asset inspection, documentation, and notifications.

Add meaningful regression tests for security boundaries, persistence changes, and behavior that can regress. Avoid tests that only repeat implementation details. Report checks actually run and any remaining limitations in the pull request.

## Documentation changes

Update the canonical Markdown guide when a workflow changes. `console/scripts/build-documentation.mjs` renders the selected guides at build time. Do not hand-edit generated HTML or duplicate the guide in a Vue component. Keep relative links valid and use `example.com` domains and placeholder IDs. Do not include private infrastructure details or real credentials.

New guides must be registered in the documentation catalog. Run the console checks and build to validate links and rendered pages. Raw HTML in Markdown is disabled.

## Submitting a change

Use a concrete title and explain the problem, resulting behavior, and validation. Preserve existing user data and migration compatibility. Call out schema or public API changes. For a vulnerability, follow [Security](SECURITY.md) instead of posting exploit details publicly.

Contributions are provided under the project's [Apache-2.0 license](LICENSE). Preserve applicable third-party notices.
