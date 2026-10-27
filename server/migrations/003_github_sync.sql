ALTER TABLE deployment_reports ADD COLUMN patch_id BIGINT;
ALTER TABLE deployment_reports ADD CONSTRAINT deployment_patch FOREIGN KEY(patch_id,release_id) REFERENCES distribution_patches(id,release_id);
CREATE TABLE github_integrations (
    app_id TEXT PRIMARY KEY REFERENCES applications_apps(id),
    repository TEXT NOT NULL,
    installation_id BIGINT NOT NULL CHECK(installation_id>0),
    connected_at TEXT NOT NULL
);
CREATE TABLE github_workflow_runs (
    deployment_id BIGINT PRIMARY KEY REFERENCES deployment_reports(id) ON DELETE CASCADE,
    run_id BIGINT NOT NULL,
    run_attempt BIGINT NOT NULL DEFAULT 0,
    status TEXT,
    conclusion TEXT,
    github_updated_at TIMESTAMPTZ,
    checked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    refresh_version BIGINT NOT NULL DEFAULT 0,
    sync_error TEXT
);
CREATE INDEX github_workflow_checked ON github_workflow_runs(checked_at);
CREATE TABLE github_deliveries(id TEXT PRIMARY KEY,received_at TIMESTAMPTZ NOT NULL DEFAULT now());
