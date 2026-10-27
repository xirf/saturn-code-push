use super::*;

fn sqlite_fixture(path: &std::path::Path) {
    let c = rusqlite::Connection::open(path).unwrap();
    c.execute_batch("PRAGMA user_version=3;
        CREATE TABLE saturn_sequence(id INTEGER PRIMARY KEY,value INTEGER);
        INSERT INTO saturn_sequence VALUES(1,9);
        CREATE TABLE applications_apps(id TEXT,display_name TEXT,created_at TEXT);
        INSERT INTO applications_apps VALUES('app','Imported','2026-01-01');
        CREATE TABLE applications_channels(id INTEGER,app_id TEXT,name TEXT);
        INSERT INTO applications_channels VALUES(1,'app','stable');
        CREATE TABLE distribution_releases(id INTEGER,app_id TEXT,version TEXT,flutter_revision TEXT,flutter_version TEXT,platforms TEXT,created_at TEXT,updated_at TEXT);
        INSERT INTO distribution_releases VALUES(2,'app','1.0.0+1','rev',NULL,'{}','2026-01-01','2026-01-01');
        CREATE TABLE distribution_patches(id INTEGER,release_id INTEGER,number INTEGER,channel_ids TEXT,is_rolled_back INTEGER,created_at TEXT);
        INSERT INTO distribution_patches VALUES(3,2,1,'[1]',0,'2026-01-01');
        CREATE TABLE artifacts_binaries(id INTEGER,app_id TEXT,release_id INTEGER,patch_id INTEGER,arch TEXT,platform TEXT,hash TEXT,hash_signature TEXT,size INTEGER,can_sideload INTEGER,signing TEXT,upload_key TEXT,download_key TEXT,uploaded INTEGER);
        INSERT INTO artifacts_binaries VALUES(9,'app',2,NULL,'x86_64','android','hash',NULL,3,0,'null','upload','download',1);
        CREATE TABLE artifacts_bundles(id INTEGER,app_id TEXT,release_id INTEGER,patch_id INTEGER,platform TEXT,sha256 TEXT,size INTEGER,upload_key TEXT,download_key TEXT,uploaded INTEGER);
        INSERT INTO artifacts_bundles VALUES(10,'app',2,3,'android','bundle-hash',3,'asset-upload','asset-download',1);
        CREATE TABLE distribution_patch_assets(patch_id INTEGER,platform TEXT);
        INSERT INTO distribution_patch_assets VALUES(3,'android');
        CREATE TABLE artifact_asset_signatures(bundle_id INTEGER,signature TEXT);
        INSERT INTO artifact_asset_signatures VALUES(10,'original-signature');
        CREATE TABLE deployment_reports(id INTEGER,app_id TEXT,release_id INTEGER,report_key TEXT,body TEXT,created_at TEXT,updated_at TEXT);
        INSERT INTO deployment_reports VALUES(4,'app',2,'run','{\"key\":\"run\",\"repository\":\"owner/repo\",\"commit\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"source_kind\":\"tag\",\"source_ref\":\"v1\",\"destination\":\"google-play/production\",\"status\":\"published\"}','2026-01-01','2026-01-01');
    ").unwrap();
}
#[test]
fn sqlite_import_preserves_ids_and_does_not_modify_source() {
    let (dir, s) = service();
    let path = dir.path().join("old.sqlite3");
    sqlite_fixture(&path);
    let before = std::fs::read(&path).unwrap();
    std::fs::write(s.objects.binary(9), b"abc").unwrap();
    crate::sqlite_import::import(&s.database, &path).unwrap();
    assert_eq!(
        s.read(|d| Ok(d.apps[0].display_name.clone())).unwrap(),
        "Imported"
    );
    assert_eq!(s.read(|d| Ok(d.releases[0].id)).unwrap(), 2);
    assert_eq!(s.read(|d| Ok(d.sequence)).unwrap(), 10);
    s.database
        .transaction(|c| {
            assert_eq!(
                c.query_one("SELECT report_key FROM deployment_reports WHERE id=4", &[])
                    .map_err(internal)?
                    .get::<_, String>(0),
                "run"
            );
            assert_eq!(crate::platform::database::next_id(c)?, 11);
            assert_eq!(
                c.query_one(
                    "SELECT signature FROM artifact_asset_signatures WHERE bundle_id=10",
                    &[]
                )
                .map_err(internal)?
                .get::<_, String>(0),
                "original-signature"
            );
            assert_eq!(
                c.query_one(
                    "SELECT platform FROM distribution_patch_assets WHERE patch_id=3",
                    &[]
                )
                .map_err(internal)?
                .get::<_, String>(0),
                "android"
            );
            Ok(())
        })
        .unwrap();
    assert_eq!(std::fs::read(s.objects.binary(9)).unwrap(), b"abc");
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(crate::sqlite_import::import(&s.database, &path).is_err());
}
#[test]
fn failed_import_rolls_back_catalog() {
    let (dir, s) = service();
    let path = dir.path().join("old.sqlite3");
    sqlite_fixture(&path);
    let c = rusqlite::Connection::open(&path).unwrap();
    c.execute("UPDATE distribution_releases SET app_id='missing'", [])
        .unwrap();
    drop(c);
    assert!(crate::sqlite_import::import(&s.database, &path).is_err());
    assert!(s.read(|d| Ok(d.apps.is_empty())).unwrap());
}
#[test]
fn newer_schema_is_rejected() {
    let (_dir, s) = service();
    s.database
        .transaction(|c| {
            c.execute("INSERT INTO saturn_schema(version) VALUES(999)", &[])
                .map_err(internal)?;
            Ok(())
        })
        .unwrap();
    assert!(s.database.transaction(crate::migration::run).is_err());
}
