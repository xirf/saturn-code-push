# Saturn console

Nuxt 4 and TypeScript, with Vue Composition API throughout. The production build is a static SPA served by Rust from `.output/public`; deployment requires no Node server.

## Ownership

```text
app/
├── pages/                   # Route composition and page metadata
├── layouts/                 # Authenticated shell
├── middleware/              # Client navigation guards
├── components/
│   ├── ui/                  # Generated shadcn-vue primitives
│   ├── brand/
│   ├── shared/
│   └── shell/
├── features/
│   ├── applications/
│   ├── distribution/
│   ├── artifacts/
│   ├── identity/
│   ├── telemetry/
│   └── workspace/           # Cross-feature snapshot query and projections
├── stores/                  # Pinia UI preferences
├── lib/                     # Same-origin transport, validation, formatting
└── assets/css/              # Official shadcn theme and base styles
```

Within a feature, keep components, composables, queries/mutations, stores, and entity types in separate folders as needed. Components render data and invoke composables; they do not issue raw HTTP requests. Entity schemas validate server responses with Zod.

Pinia Colada owns remote session and catalog data, loading/errors, mutations, and invalidation. Pinia stores only application selection, mobile navigation, reporting-period preferences, and the session cache generation. Do not duplicate server responses in ordinary stores.

## Components and styles

The shadcn-vue 2.8.2 CLI generated the `reka-nova` components and neutral theme recorded in `components.json`. Use those primitives for buttons, dialogs, dropdown menus, selects, sheets, tables, and alerts. Application selection and channel selection use `Select`; patch actions use `DropdownMenu`, with a separate confirmation `Dialog`.

Use the generated semantic tokens such as `background`, `foreground`, `muted`, `border`, and `primary`. Use Tailwind utilities for page layout. Do not add parallel CSS rules that restyle library controls, menus, or table primitives.

`app/assets/css/shadcn.css` is the official `shadcn-vue@2.8.2/dist/tailwind.css`, vendored with its MIT license in `public/licenses/shadcn-vue.txt`. Keeping this CSS in the repository avoids installing the CLI and its code-transformation dependency tree during every application build. `main.css` contains the CLI-generated theme; its fonts are the locally bundled DM Sans and Manrope packages. The Sonner component also imports its required vue-sonner stylesheet in app.vue. When refreshing the registry components, refresh the official base stylesheet and license at the same time.

Prettier expands Vue attributes and templates. Run `npm run format` after edits and `npm run check` before building. The Docker build enforces the formatting check, ESLint, type checking, and unit tests.

## Container development

From the repository root:

```sh
docker compose -f console/compose.dev.yaml up
```

Nuxt listens at `http://localhost:3001`. Set `SATURN_API_ORIGIN` to a separately configured development API if it is not reachable through `host.docker.internal:3000`. For interactive sign-in, that API's `PUBLIC_BASE_URL` must match the browser origin, for example `http://localhost:3001`. The development proxy preserves the browser's Origin header; it does not bypass the server's origin checks. Keep production credentials and data separate.

Run frontend checks without starting the development server:

```sh
docker compose -f console/compose.dev.yaml run --rm console sh -c 'npm ci && npm run check'
```

Docker builds both the console and Rust server using `server/Dockerfile`. Podman can build and run the same image; production configuration remains in `server/compose.yaml`.

## Browser validation

```sh
bash scripts/test-console.sh
```

This builds the application and Linux Playwright runner, creates a temporary network, and starts a server with temporary credentials and a RAM-backed data directory. It publishes no host ports and never mounts the production data volume. Containers and the network are removed when the script exits. Set `CONTAINER_ENGINE=podman` to run it with Podman.

Tests cover sign-in errors, application creation, deep-link reloads, application selection, patch promotion/rollback/restore, dropdown sizing, mobile navigation, session expiry, logout cancellation, browser storage, and CSP violations. Screenshots and traces go to ignored `console/test-results/`; the HTML report goes to ignored `console/playwright-report/`.

## Session and script security

The API transport uses same-origin cookies and rejects external URLs. No bearer token or session token is stored in localStorage, sessionStorage, or Pinia. Login passwords are not passed as mutation variables and are cleared after the request. Logout and authentication changes cancel private queries, remove private query and mutation entries, and advance the cache generation so stale responses cannot populate a later session.

Client navigation guards improve the interface; Rust still authorizes every management request. A 401 from the workspace query clears the local session and returns to sign-in.

Zod is configured centrally with `jitless: true` before schemas are constructed. Nuxt import maps are disabled. The static-build postprocessor uses `parse5` to externalize executable bootstrap scripts into local files; the server retains `script-src 'self'` without `unsafe-eval` or executable inline-script exceptions. Browser tests verify that the deployed page does not violate this policy.

## Dependency audit limitation

At migration validation, `npm audit` still reports the unpatched [node-forge signature-verification advisory](https://github.com/advisories/GHSA-86w9-cpqp-85rv) through Nuxt's `listhen` development HTTPS dependency. This is not a clean dependency audit. Development HTTPS is disabled, and the production image contains compiled browser assets and Rust, with no Node runtime or `node-forge` package. Do not enable the affected development HTTPS tooling without reassessing the advisory. Recheck the upstream fix when updating Nuxt; the audit is not suppressed or force-fixed by downgrading the framework.

## Inspect patch assets

Open **Patches** and select **Inspect** in the **Assets** column. The dialog provides a searchable file list, bounded image/text previews, platform selection, the bundle SHA-256, and an authenticated ZIP download. Binary manifests and fonts show metadata. See the [asset update guide](../docs/asset-support.md) for the full publishing, inspection, testing, and promotion workflow. [Using Saturn with Shorebird](../docs/usage.md) explains which steps differ from regular hosted Shorebird.

## Public web documentation

The public `/docs` routes use a separate documentation layout and require no administrator session. The console navigation, login screen, and asset inspector link to them. Markdown in the repository remains the canonical source.

`npm run docs:generate` uses `scripts/documentation/catalog.mjs` to find guides, then renders Markdown with raw HTML disabled. It rewrites relative guide links to web routes, validates heading targets, and generates `app/generated/documentation.json`. Never edit that generated file. The generator runs before development, type checking, and builds. Docker copies the selected source guides into the build stage; Compose mounts the repository read-only for documentation generation.

Add or edit guides in their source Markdown, register new pages in the catalog, and run `npm run check` and `npm run build`. Restart the development command after changing Markdown to regenerate its content. The documentation renderer uses Markdown-It and GitHub-compatible heading slugs; Tailwind Typography supplies the article styles. Code-copy buttons use the same Sonner notifications as the console.

The typography selector-parser override selects a patched compatible 6.x version; keep it until the plugin updates its dependency range. Existing third-party license texts remain in `public/licenses/`.
