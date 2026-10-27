-- Absence of a row means the original flat local-disk layout.
CREATE TABLE artifact_objects (
    id BIGSERIAL PRIMARY KEY,
    binary_id BIGINT UNIQUE REFERENCES artifacts_binaries(id),
    bundle_id BIGINT UNIQUE REFERENCES artifacts_bundles(id),
    object_key TEXT NOT NULL,
    bucket TEXT,
    endpoint TEXT,
    region TEXT,
    CHECK ((binary_id IS NOT NULL)::int + (bundle_id IS NOT NULL)::int = 1),
    CHECK ((bucket IS NULL AND endpoint IS NULL AND region IS NULL)
        OR (bucket IS NOT NULL AND endpoint IS NOT NULL AND region IS NOT NULL))
);
