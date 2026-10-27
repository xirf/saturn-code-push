use super::{
    database::Database,
    error::{Result, bad, internal},
};
use postgres::Transaction;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
pub(crate) mod s3;
pub(crate) use s3::S3;

#[derive(Clone, Copy)]
pub(crate) enum Kind {
    Binary,
    Bundle,
}
impl Kind {
    fn column(self) -> &'static str {
        match self {
            Self::Binary => "binary_id",
            Self::Bundle => "bundle_id",
        }
    }
}
#[derive(Clone)]
pub(crate) struct Location {
    pub key: String,
    pub remote: Option<Arc<S3>>,
}
impl Location {
    pub fn save(&self, c: &mut Transaction<'_>, kind: Kind, id: i64) -> Result<()> {
        let (bucket, endpoint, region) = match &self.remote {
            Some(s) => (Some(&s.bucket), Some(&s.endpoint), Some(&s.region)),
            None => (None, None, None),
        };
        c.execute(&format!("INSERT INTO artifact_objects({},object_key,bucket,endpoint,region) VALUES($1,$2,$3,$4,$5)",kind.column()), &[&id,&self.key,&bucket,&endpoint,&region]).map_err(internal)?;
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct ObjectStore {
    root: PathBuf,
    s3: Option<Arc<S3>>,
    use_s3: bool,
}
/// Keeps a temporary remote download alive through inspection or opening ServeFile.
pub(crate) struct LocalObject {
    pub path: PathBuf,
    _temporary: Option<tempfile::NamedTempFile>,
}
impl ObjectStore {
    pub fn open(root: &Path) -> Result<Self> {
        let root = root.join("objects");
        std::fs::create_dir_all(&root).map_err(internal)?;
        Ok(Self {
            root,
            s3: None,
            use_s3: false,
        })
    }
    pub fn settings(&self) -> serde_json::Value {
        serde_json::json!({
            "backend": if self.use_s3 { "s3" } else { "local" },
            "s3": self.s3.as_ref().map(|s| serde_json::json!({"bucket":s.bucket,"endpoint":s.endpoint,"region":s.region,"prefix":s.prefix})),
        })
    }
    pub fn configure(&mut self, s3: Option<Arc<S3>>, use_s3: bool) {
        self.s3 = s3;
        self.use_s3 = use_s3;
    }
    // Legacy paths remain supported for existing databases and SQLite imports.
    #[cfg(test)]
    pub fn binary(&self, id: i64) -> PathBuf {
        self.root.join(id.to_string())
    }
    #[cfg(test)]
    pub fn bundle(&self, id: i64) -> PathBuf {
        self.root.join(format!("asset-{id}"))
    }
    pub fn directory(&self) -> &Path {
        &self.root
    }
    pub fn destination(
        &self,
        kind: Kind,
        app: &str,
        release: i64,
        patch: Option<i64>,
        id: i64,
    ) -> Location {
        // Encode every special character so app IDs cannot become filesystem paths.
        let app = percent_encoding::utf8_percent_encode(app, percent_encoding::NON_ALPHANUMERIC);
        let mut key = format!("apps/{app}/releases/{release}");
        if let Some(patch) = patch {
            key.push_str(&format!("/patches/{patch}"));
        }
        key.push_str(match kind {
            Kind::Binary => "/binaries/",
            Kind::Bundle => "/assets/",
        });
        key.push_str(&id.to_string());
        let remote = self
            .use_s3
            .then(|| self.s3.clone().expect("validated S3 configuration"));
        if let Some(s3) = &remote {
            // Retries and concurrent requests never overwrite another attempt's bytes.
            key.push_str(&format!("/{}", uuid::Uuid::new_v4()));
            if !s3.prefix.is_empty() {
                key = format!("{}/{key}", s3.prefix);
            }
        }
        Location { key, remote }
    }
    pub fn local_path(&self, location: &Location) -> PathBuf {
        self.root.join(&location.key)
    }
    pub async fn location(&self, database: &Database, kind: Kind, id: i64) -> Result<Location> {
        let row = database.call(move |c| c.query_opt(&format!("SELECT object_key,bucket,endpoint,region FROM artifact_objects WHERE {}=$1", kind.column()), &[&id]).map_err(internal)).await?;
        let Some(row) = row else {
            return Ok(Location {
                key: match kind {
                    Kind::Binary => id.to_string(),
                    Kind::Bundle => format!("asset-{id}"),
                },
                remote: None,
            });
        };
        let bucket: Option<String> = row.get(1);
        let remote = if let Some(bucket) = bucket {
            let s3 = self
                .s3
                .clone()
                .ok_or_else(|| internal("S3 is required to read this artifact"))?;
            if s3.bucket != bucket
                || s3.endpoint != row.get::<_, String>(2)
                || s3.region != row.get::<_, String>(3)
            {
                return Err(internal(
                    "S3 configuration does not match the artifact storage location",
                ));
            }
            Some(s3)
        } else {
            None
        };
        Ok(Location {
            key: row.get(0),
            remote,
        })
    }
    pub async fn materialize(
        &self,
        database: &Database,
        kind: Kind,
        id: i64,
    ) -> Result<LocalObject> {
        let location = self.location(database, kind, id).await?;
        if let Some(s3) = &location.remote {
            let temp = s3.fetch(&location.key, &self.root).await?;
            Ok(LocalObject {
                path: temp.path().into(),
                _temporary: Some(temp),
            })
        } else {
            Ok(LocalObject {
                path: self.local_path(&location),
                _temporary: None,
            })
        }
    }
}
