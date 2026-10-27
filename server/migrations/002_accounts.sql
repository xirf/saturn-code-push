CREATE TABLE identity_accounts (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE CHECK(length(username) BETWEEN 1 AND 128),
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('admin','publisher','viewer')),
    disabled BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
-- Old single-operator sessions do not establish a database account identity.
DELETE FROM dashboard_sessions;
ALTER TABLE dashboard_sessions ADD CONSTRAINT session_account FOREIGN KEY(identity) REFERENCES identity_accounts(id);
CREATE INDEX session_expiry ON dashboard_sessions(expires_at);
CREATE TABLE identity_api_keys (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    prefix TEXT NOT NULL,
    app_id TEXT NOT NULL REFERENCES applications_apps(id),
    permission TEXT NOT NULL CHECK(permission IN ('deployments:write','publish')),
    created_by TEXT NOT NULL REFERENCES identity_accounts(id),
    created_at TEXT NOT NULL,
    expires_at BIGINT NOT NULL,
    revoked_at TEXT,
    last_used_at TEXT
);
CREATE TABLE identity_audit (
    id BIGSERIAL PRIMARY KEY,
    actor_id TEXT NOT NULL,
    actor_name TEXT NOT NULL,
    actor_kind TEXT NOT NULL CHECK(actor_kind IN ('account','api_key','legacy_token','system')),
    action TEXT NOT NULL,
    resource TEXT NOT NULL,
    app_id TEXT,
    occurred_at TEXT NOT NULL
);
CREATE INDEX identity_audit_app ON identity_audit(app_id, id DESC);
ALTER TABLE deployment_reports ADD CONSTRAINT deployment_release FOREIGN KEY(release_id,app_id) REFERENCES distribution_releases(id,app_id);
ALTER TABLE deployment_reports ADD COLUMN reported_by TEXT;
ALTER TABLE deployment_reports ADD COLUMN updated_by TEXT;

CREATE TABLE saturn_imports (source TEXT NOT NULL, imported_at TIMESTAMPTZ NOT NULL DEFAULT now());
