# GitHub deployment reporting

Open a release in the console and use **Deployments & CI/CD → Link a deployment manually** to record its repository, full commit SHA, branch/tag, destination and reported status. Build number and GitHub Actions run ID are optional. Use a stable destination such as `app-store`, `testflight`, or `production`.

Statuses are independent of release activation: `pending`, `running`, `failed`, `uploaded`, `in_review`, `published`. A successful workflow or upload does not establish that an app is live. Set `published` only after confirming publication. App Store Connect status is not synced.

The default comparison is the previous recorded deployment marked `published` for the same destination and repository, ordered by initial report ID. You can select another recorded deployment from the same app as the baseline. GitHub opens the commit comparison; the console does not invent a commit count or fetch private repository contents. Users need their own GitHub repository access.

## Distinguish a release from a patch

The **Saturn target** and **GitHub source** are different fields:

| Report | Required target | Meaning |
| --- | --- | --- |
| Release baseline | `release_id`, omit `patch_id` | The baseline binary/store build registered for this release |
| Patch | `release_id` plus `patch_id` | One existing patch under that exact release |

The dashboard's **Deployment target** selector offers the release baseline and its patches. The server verifies app/release/patch ownership and prevents an existing deployment key from being reassigned to another target. A GitHub tag or Release trigger does not determine whether the Saturn target is a release or patch.

For patches, the default comparison uses the previous published patch under the same release, repository, and destination, falling back to that release's published baseline. Store release comparisons skip patch reports. Explicitly selected comparison baselines remain available.

## Connect a GitHub App

GitHub login is not required. Existing Saturn accounts and roles continue to control access. An administrator connects a GitHub App installation to one repository per Saturn app; monorepo installations may be connected to multiple Saturn apps.

1. Register a GitHub App owned by the person or organization operating this Saturn instance. Request **Actions: Read-only** and the mandatory **Metadata: Read-only** repository permissions. Subscribe to **Workflow run** events. No code-write, workflow-execution, or user OAuth permissions are needed.
2. Set its webhook URL to `https://updates.example.com/integrations/github/webhook`, enable webhook delivery, and set a random secret of at least 32 bytes.
3. Generate/download an RSA private key. Supply these values through deployment settings and restart Saturn:

   | Variable | Value |
   | --- | --- |
   | `GITHUB_APP_ID` | Numeric GitHub App ID |
   | `GITHUB_APP_SLUG` | App slug from its GitHub URL |
   | `GITHUB_APP_PRIVATE_KEY_FILE` | Container path to its read-only mounted PEM private key |
   | `GITHUB_WEBHOOK_SECRET` | The same secret configured on the GitHub App |

   Keep the private key and secret out of Git and browser inputs. The server runs as UID 10001; grant that user read access to the mounted key. The App private key grants access to its installations, so use an App controlled by this instance's operator.
4. Open **Settings → GitHub integration → Install GitHub App**, choose the repositories it may access, and complete installation on GitHub.
5. Copy the installation ID from the installation settings URL (`.../settings/installations/12345`). Back in Saturn, choose the application and enter `owner/repository` plus that ID. **Connect repository** verifies the configured App installation and repository access with GitHub before saving the association.

Hosting platforms can mount their managed secrets directly. For local Compose, an optional override mounts an existing host key without copying it into the image:

```bash
# Also set GITHUB_APP_PRIVATE_KEY_HOST_FILE in deployment settings/server/.env.
docker compose --env-file server/.env -f server/compose.yaml -f server/compose.github.yaml up -d --build
```

The server uses short-lived App JWTs and repository-scoped installation tokens; tokens stay in memory and are never stored in the database or returned to the dashboard. Redirects are disabled for authenticated GitHub requests. Outbound HTTPS access to `api.github.com` is required.

## Workflow status synchronization

Link a run ID when reporting a release or patch. Signed webhooks wake the synchronizer, which fetches authoritative run state from GitHub. Delivery IDs are deduplicated for seven days; replayed or out-of-order webhook payloads cannot directly overwrite workflow or deployment state. Missing events are recovered by periodic reconciliation.

Active/error runs are eligible every 60 seconds; completed runs are rechecked hourly. The reporting helper pins `workflow_attempt` from `GITHUB_RUN_ATTEMPT`, so a later rerun does not rewrite the CI provenance of an already-deployed build. Manual/legacy reports with no attempt track the latest run attempt and discover reruns during reconciliation. Each batch processes up to 100 due reports, reuses scoped tokens within the batch, and yields on network/API failures before retrying later. **Refresh linked runs** queues an earlier reconciliation. The release page refreshes visible deployment history every 15 seconds.

The fetched run must match the connected repository, run ID, and reported commit SHA. Newer run attempts supersede older ones. The dashboard displays CI status/conclusion/attempt independently of reported deployment status. **CI success never activates a release, promotes a patch, or establishes store publication.** Failed synchronization displays an error and identifies the last known CI state as potentially stale. Replacing repository/commit/run metadata clears an old CI snapshot. Disconnecting removes snapshots and stops synchronization; uninstalling or suspending the App results in an access error until access is restored or the connection is removed.

## Report from GitHub Actions

Copy `scripts/report-deployment.py` into your app repository. Configure `SATURN_URL`, `SATURN_APP_ID` and `SATURN_RELEASE_ID` as Actions variables and `SATURN_TOKEN` as an Actions secret containing an app-scoped **Report deployments** API key created in Settings. Use a key for this application with `deployments:write` permission and an appropriate expiry. The server must be reachable from the runner over HTTPS.

After the workflow has created the Saturn release and uploaded the app, add:

```yaml
- name: Report uploaded build
  env:
    SATURN_URL: ${{ vars.SATURN_URL }}
    SATURN_APP_ID: ${{ vars.SATURN_APP_ID }}
    SATURN_TOKEN: ${{ secrets.SATURN_TOKEN }}
    SATURN_RELEASE_ID: ${{ vars.SATURN_RELEASE_ID }}
  run: >-
    python3 scripts/report-deployment.py
    --release-id "$SATURN_RELEASE_ID"
    --destination testflight
    --status uploaded
    --build-number "$GITHUB_RUN_NUMBER"
```

Use your actual app build number if it differs from the Actions run number. Supply the release ID from your release-creation step for dynamically created releases. The helper reads the repository, source ref, SHA and run ID from GitHub's environment. Tag and release triggers are detected automatically; `--source-kind` can override detection. `GITHUB_SHA` must describe the code actually built; override it when checking out a different commit.

Call the helper at earlier/later steps with `running`, `failed` or `published` as appropriate. The key includes repository, run ID, run attempt, destination, and release/patch target, so repeated calls in one attempt update one record; reruns create separate history. Additional matrix jobs targeting the same destination need distinct destination names or a custom reporting client.

For a patch, use the same parent release ID and pass the patch ID returned by your patch-creation step:

```bash
python3 scripts/report-deployment.py \
  --release-id "$SATURN_RELEASE_ID" \
  --patch-id "$SATURN_PATCH_ID" \
  --destination ota/stable \
  --status uploaded
```

Report `published` only after the actual release publication or patch-promotion step succeeds. Separate jobs in the same run can report different release and patch targets without overwriting each other.

## HTTP API

See [accounts and PostgreSQL](accounts-and-postgresql.md) for key management and publisher attribution.

Management authentication applies: a dashboard session with same-origin mutation protection, or `Authorization: Bearer <management-token>` for CI.

- `GET /api/v1/apps/{app}/deployments` lists records newest first.
- `POST /api/v1/apps/{app}/deployments` inserts or replaces a report by app-scoped `key`. Body: `release_id`, optional `patch_id`, `key`, `repository` (`owner/repo`), `commit` (full SHA), `source_kind` (`commit`, `tag`, `release`), `source_ref`, `destination`, `status`, optional `build_number`, `workflow_run` (positive integer), and `workflow_attempt` (positive integer requiring a run ID; omit to follow the latest attempt).
- `PATCH /api/v1/apps/{app}/deployments/{id}` updates only `status`.

A key cannot be reassigned to a different release or patch. Deployment responses include nullable `patch_id` and `workflow` status objects. Reports are supplied metadata, not cryptographic attestations. Manual/CI reporting works without a GitHub App. Live workflow syncing requires the GitHub App configuration above. No GitHub login, workflow execution, or store API integration is required.
