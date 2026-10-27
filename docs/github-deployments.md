# GitHub deployment reporting

Open a release in the console and use **Deployments & CI/CD → Link a deployment manually** to record its repository, full commit SHA, branch/tag, destination and reported status. Build number and GitHub Actions run ID are optional. Use a stable destination such as `app-store`, `testflight`, or `production`.

Statuses are independent of release activation: `pending`, `running`, `failed`, `uploaded`, `in_review`, `published`. A successful workflow or upload does not establish that an app is live. Set `published` only after confirming publication. App Store Connect status is not synced.

The default comparison is the previous recorded deployment marked `published` for the same destination and repository, ordered by initial report ID. You can select another recorded deployment from the same app as the baseline. GitHub opens the commit comparison; the console does not invent a commit count or fetch private repository contents. Users need their own GitHub repository access.

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

Call the helper at earlier/later steps with `running`, `failed` or `published` as appropriate. The key includes repository, run ID, run attempt and destination, so repeated calls in one attempt update one record; reruns create separate history. Additional matrix jobs targeting the same destination need distinct destination names or a custom reporting client.

## HTTP API

See [accounts and PostgreSQL](accounts-and-postgresql.md) for key management and publisher attribution.

Management authentication applies: a dashboard session with same-origin mutation protection, or `Authorization: Bearer <management-token>` for CI.

- `GET /api/v1/apps/{app}/deployments` lists records newest first.
- `POST /api/v1/apps/{app}/deployments` inserts or replaces a report by app-scoped `key`. Body: `release_id`, `key`, `repository` (`owner/repo`), `commit` (full SHA), `source_kind` (`commit`, `tag`, `release`), `source_ref`, `destination`, `status`, optional `build_number` and `workflow_run` (positive integer).
- `PATCH /api/v1/apps/{app}/deployments/{id}` updates only `status`.

A key cannot be reassigned to a different release. Reports are supplied metadata, not cryptographic attestations. No GitHub OAuth, webhook setup, workflow execution, or App Store integration is required.
