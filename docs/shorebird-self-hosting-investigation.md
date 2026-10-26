# Shorebird self-hosting investigation

Initial feasibility study: 2026-10-26. The subsequent implementation and successful device proof are recorded in [Android validation](android-validation.md). The findings below describe the initial study.

## Result

A replacement patch server and an independently built dashboard appear technically feasible from the public client contracts. This is an engineering inference, not demonstrated compatibility. Shorebird does not currently offer an official self-hosted distribution ([security policy](https://docs.shorebird.dev/system/security/)).

Replacing the hosted API does not establish an independently buildable Shorebird toolchain. Its architecture documentation identifies its Dart SDK fork as private. In particular, iOS patch generation depends on modified compiler/interpreter/linker behavior. A server replacement could still depend on Shorebird-distributed build artifacts ([architecture](https://docs.shorebird.dev/code-push/system-architecture/)).

The public monorepo tree inspected contains CLI, client and protocol code, but no server implementation or console application. This is a bounded search of that repository, not proof that no implementation exists elsewhere. The public console redirects to sign-in; authenticated screens, browser traffic and dashboard behavior were not inspected ([console](https://console.shorebird.dev/)).

## REA identification and use

The requested project is [morluto/rea — Reverse Engineer Anything](https://github.com/morluto/rea). Its bundled skill is named [reverse-engineer-anything](https://github.com/morluto/rea/blob/main/.agents/skills/reverse-engineer-anything/SKILL.md). It supplies a CLI/MCP toolkit as well as agent instructions.

I read the skill. It explicitly routes ordinary source-repository architecture analysis to normal source tools. This investigation therefore used the GitHub connector and public documentation. REA was not installed, registered, or executed, and no decompilation was performed. Its tools cannot recover a remote server's implementation from a client alone.

If runtime dashboard investigation becomes necessary, an authenticated test account or a sanitized recording of the intended workflows would supply the missing evidence. REA could then help analyze those browser interactions. No authenticated dashboard evidence was available in this pass.

## Evidence identity and limits

GitHub connector repository tree identities returned during this investigation:

| Repository | Tree commit |
| --- | --- |
| `shorebirdtech/shorebird` | `2e68da6bd82bc21b2307ef95e41d9f00b918e668` |
| `shorebirdtech/updater` | `8f66c95db455f5b0d3d48b526feb73a4df1b6fe3` |
| `morluto/rea` | `1173c0a9dc1b352a60943a491462294db8f353ae` |

Individual files were read from `main`, not an immutable checkout; these identities record the inventory and do not guarantee an atomic snapshot of every file. A compatibility implementation must pin and fetch a single revision first. Browser search returned some older file snapshots, so the connector's direct file contents were used for the detailed API notes below.

Local Git clones failed because the configured proxy at localhost was unreachable. GitHub connector reads succeeded. No production API mutations, login attempts, or account changes were made.

## Endpoint configuration: observed

The CLI resolves its API target from `SHOREBIRD_HOSTED_URL`, then hidden dev mode, then `base_url` in `shorebird.yaml`. Auth has a separate `AUTH_SERVICE_URL` override. Artifact mirrors have their own configuration. These are separate dependencies; setting the API URL alone does not replace authentication or compiler downloads ([CLI environment source](https://github.com/shorebirdtech/shorebird/blob/main/packages/shorebird_cli/lib/src/shorebird_env.dart)).

The updater uses the YAML `base_url` when present and otherwise defaults to Shorebird's API ([updater config](https://github.com/shorebirdtech/updater/blob/main/library/src/config.rs)). Example configuration for a future compatible deployment:

```yaml
app_id: <app-uuid-created-by-our-server>
base_url: https://updates.example.com
```

This configuration is not sufficient by itself. The installed app needs the Shorebird-enabled engine and the bundled configuration. The repository currently has no `shorebird.yaml`, and no Shorebird setup was introduced into the customer app during this investigation. Existing ordinary Flutter builds would need a new store release containing an enabled engine before this patch system could update them.

## Device API contract: observed

Source: [updater network implementation](https://github.com/shorebirdtech/updater/blob/main/library/src/network.rs).

| Method and path | Purpose |
| --- | --- |
| `POST /api/v1/patches/check` | Select patch and communicate rolled-back patch numbers |
| `POST /api/v1/patches/events` | Receive device patch events |

Patch selection receives app ID, channel, release version, platform, architecture, anonymous install ID, and optionally current patch number. Its response contains `patch_available`, optional patch metadata, and optional `rolled_back_patch_numbers`. Patch metadata includes number, hash, download URL and optional hash signature.

The updater's `hash` represents the final reconstructed, uncompressed artifact. It must not be confused with the downloaded diff's hash. The CLI computes and optionally signs the full target artifact before generating the diff ([patcher implementation](https://github.com/shorebirdtech/shorebird/blob/main/packages/shorebird_cli/lib/src/commands/patch/patcher.dart)).

## Management API: observed surface, incomplete specification

The [public API client](https://github.com/shorebirdtech/shorebird/blob/main/packages/shorebird_code_push_client/lib/src/code_push_client.dart) exposes these relevant route families under `/api/v1`:

| Route family | Work required |
| --- | --- |
| `/users/me`, `/organizations`, `/apps` | Identity, ownership and app metadata |
| `/apps/{app}/channels` | Channel records |
| `/apps/{app}/releases` | Release creation/listing/status |
| `/apps/{app}/releases/{release}/artifacts` | Release artifact registration and lookup |
| `/apps/{app}/patches` and patch artifact routes | Patch creation and artifact registration |
| `/apps/{app}/patches/promote` | Channel promotion |
| Release patch rollback/rollforward routes | Patch lifecycle changes |
| `/apps/{app}/signing-key` | Signing-key lookup |
| App/release metrics routes | Analytics queries |

These paths establish API scope, not complete compatibility: request/response models, errors, authentication, upload methods and state transitions still require contract fixtures. The client registers artifacts before uploading bytes to a returned URL; a generic upload endpoint will not automatically satisfy its storage protocol.

## Proposed replacement design

These are design choices for our implementation, not observations about Shorebird internals:

- A management API with authenticated app/release/patch/channel operations.
- A device-facing API for patch checks and events, with rate limits and bounded payloads.
- PostgreSQL for ownership, release state, artifact metadata, channel assignments and rollback records.
- Object storage for exact release baselines and immutable patch diffs, with private release access and controlled patch downloads.
- A worker for artifact validation and event aggregation.
- An original dashboard for apps, releases, architecture coverage, promotion, rollback, signing status and download/install/failure metrics.
- Container deployment with persistent storage, TLS, backups and protected signing keys.

Patch eligibility must match app, release version, platform, architecture and channel. A patch should become visible only after its required artifacts have been verified. Publication and rollback need transactionally consistent state. Device events are client-reported observations; they must not authorize publishing or be treated as unquestionable proof of device state.

The dashboard can implement the needed functions without duplicating Shorebird's private UI. Exact screen parity and metric definitions remain unobserved.

## Validation gates before use

1. Pin a CLI/updater revision and extract request, response and error fixtures from that version. Cover bootstrap/auth, signing and storage protocols as well as route names.
2. Build one Android architecture with an enabled engine and our endpoint. Publish a baseline and one genuine binary patch. Demonstrate download, reconstruction, verification and restart into the patched code on a test device.
3. Exercise no-update, wrong-release, wrong-architecture, interrupted-download, corrupt-artifact and bad-signature cases. Demonstrate server rollback and local failed-launch recovery.
4. Build the dashboard against the same working backend. Verify promotion and rollback affect device checks, and metrics reflect the collected events.
5. Validate further architectures and iOS separately. Inventory every required compiler/engine/diff artifact and establish which can be independently rebuilt, mirrored or must remain externally supplied.

## Outstanding questions

| Question | Status |
| --- | --- |
| Does REA exist, and where is its skill? | Answered |
| Can the public clients target another API? | Answered from source; runtime untested |
| Can official Shorebird server/dashboard code simply be deployed? | No official self-hosted distribution found; implementation absent from inspected monorepo |
| Can a compatible replacement be built? | Plausible from contracts; not yet demonstrated |
| Is the entire toolchain independently reproducible? | Unresolved; documented private Dart fork is a material dependency |
| Has the authenticated dashboard been reverse engineered? | No |
| Is this app ready for self-hosted updates? | No; no integration or device validation performed |

The next executable milestone is an Android release-to-patch-to-rollback compatibility proof. A dashboard should be built on that demonstrated backend rather than used as evidence that patch delivery works.

