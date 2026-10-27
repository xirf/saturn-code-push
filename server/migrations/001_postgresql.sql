CREATE TABLE IF NOT EXISTS applications_apps (
                id TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

CREATE TABLE IF NOT EXISTS applications_channels (
                id BIGINT PRIMARY KEY,
                app_id TEXT NOT NULL,
                name TEXT NOT NULL,
                FOREIGN KEY(app_id) REFERENCES applications_apps(id),
                UNIQUE(app_id,name)
            );

CREATE TABLE IF NOT EXISTS distribution_releases (
                id BIGINT PRIMARY KEY,
                app_id TEXT NOT NULL,
                version TEXT NOT NULL,
                flutter_revision TEXT NOT NULL,
                flutter_version TEXT,
                platforms TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY(app_id) REFERENCES applications_apps(id),
                UNIQUE(app_id,version),
                UNIQUE(id,app_id)
            );

CREATE TABLE IF NOT EXISTS distribution_patches (
                id BIGINT PRIMARY KEY,
                release_id BIGINT NOT NULL,
                number BIGINT NOT NULL,
                channel_ids TEXT NOT NULL,
                is_rolled_back BOOLEAN NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(release_id) REFERENCES distribution_releases(id),
                UNIQUE(release_id,number),
                UNIQUE(id,release_id)
            );
            CREATE TABLE IF NOT EXISTS distribution_patch_assets (
                patch_id BIGINT NOT NULL REFERENCES distribution_patches(id),
                platform TEXT NOT NULL,
                PRIMARY KEY(patch_id, platform)
            );

CREATE TABLE IF NOT EXISTS artifacts_binaries (
                id BIGINT PRIMARY KEY,
                app_id TEXT NOT NULL,
                release_id BIGINT NOT NULL,
                patch_id BIGINT,
                arch TEXT NOT NULL,
                platform TEXT NOT NULL,
                hash TEXT NOT NULL,
                hash_signature TEXT,
                size BIGINT NOT NULL CHECK (size > 0),
                can_sideload BOOLEAN NOT NULL,
                signing TEXT,
                upload_key TEXT NOT NULL,
                download_key TEXT NOT NULL,
                uploaded BOOLEAN NOT NULL,
                FOREIGN KEY(app_id) REFERENCES applications_apps(id),
                FOREIGN KEY(release_id,app_id) REFERENCES distribution_releases(id,app_id),
                FOREIGN KEY(patch_id,release_id) REFERENCES distribution_patches(id,release_id),
                UNIQUE(upload_key),
                UNIQUE(download_key)
            );
            CREATE UNIQUE INDEX IF NOT EXISTS artifacts_release_target
                ON artifacts_binaries(release_id,arch,platform) WHERE patch_id IS NULL;
            CREATE UNIQUE INDEX IF NOT EXISTS artifacts_patch_target
                ON artifacts_binaries(patch_id,arch,platform) WHERE patch_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS artifacts_bundles (
                id BIGINT PRIMARY KEY,
                app_id TEXT NOT NULL,
                release_id BIGINT NOT NULL,
                patch_id BIGINT NOT NULL,
                platform TEXT NOT NULL,
                sha256 TEXT NOT NULL,
                size BIGINT NOT NULL CHECK (size > 0),
                upload_key TEXT NOT NULL,
                download_key TEXT NOT NULL,
                uploaded BOOLEAN NOT NULL,
                FOREIGN KEY(app_id) REFERENCES applications_apps(id),
                FOREIGN KEY(release_id,app_id) REFERENCES distribution_releases(id,app_id),
                FOREIGN KEY(patch_id,release_id) REFERENCES distribution_patches(id,release_id),
                UNIQUE(patch_id,platform),
                UNIQUE(upload_key),
                UNIQUE(download_key)
            );

CREATE TABLE IF NOT EXISTS artifact_asset_signatures (bundle_id BIGINT PRIMARY KEY, signature TEXT NOT NULL);

CREATE TABLE IF NOT EXISTS telemetry_events (
            id BIGSERIAL PRIMARY KEY,
            app_id TEXT NOT NULL REFERENCES applications_apps(id),
            received_at TEXT NOT NULL,
            event TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS telemetry_events_app ON telemetry_events(app_id);
        CREATE INDEX IF NOT EXISTS telemetry_events_identity ON telemetry_events(app_id, (event::jsonb ->> 'event_id'));

CREATE TABLE IF NOT EXISTS dashboard_sessions (
            token_hash TEXT PRIMARY KEY,
            identity TEXT NOT NULL,
            expires_at BIGINT NOT NULL
        );

CREATE TABLE IF NOT EXISTS deployment_reports (
        id BIGSERIAL PRIMARY KEY, app_id TEXT NOT NULL, release_id BIGINT NOT NULL,
        report_key TEXT NOT NULL, body TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
        UNIQUE(app_id, report_key));

CREATE TABLE saturn_sequence (id BIGINT PRIMARY KEY CHECK(id=1), value BIGINT NOT NULL); INSERT INTO saturn_sequence VALUES(1,0);