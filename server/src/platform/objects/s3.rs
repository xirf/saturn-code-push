use super::*;
use object_store::{
    ClientOptions, ObjectStoreExt, WriteMultipart,
    aws::{AmazonS3, AmazonS3Builder},
    signer::Signer,
};
use std::time::Duration;
use tokio::io::AsyncReadExt;

pub(crate) struct S3 {
    pub bucket: String,
    pub endpoint: String,
    pub region: String,
    pub prefix: String,
    client: AmazonS3,
}

impl S3 {
    pub fn from_env() -> Result<Option<Arc<Self>>> {
        let bucket = std::env::var("STORAGE_S3_BUCKET").unwrap_or_default();
        if bucket.is_empty() {
            return Ok(None);
        }
        Self::build(bucket).map(|s| Some(Arc::new(s)))
    }
    #[cfg(test)]
    pub fn test_bucket(bucket: String) -> Result<Arc<Self>> {
        Self::build(bucket).map(Arc::new)
    }
    fn build(bucket: String) -> Result<Self> {
        let region =
            std::env::var("STORAGE_S3_REGION").map_err(|_| bad("STORAGE_S3_REGION is required"))?;
        let endpoint = std::env::var("STORAGE_S3_ENDPOINT")
            .ok()
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| format!("https://s3.{region}.amazonaws.com"));
        let allow_http = std::env::var("STORAGE_S3_ALLOW_HTTP").as_deref() == Ok("true");
        let url = reqwest::Url::parse(&endpoint).map_err(|_| bad("Invalid STORAGE_S3_ENDPOINT"))?;
        if url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || url.path() != "/"
            || !(url.scheme() == "https" || (allow_http && url.scheme() == "http"))
        {
            return Err(bad(
                "STORAGE_S3_ENDPOINT must be an HTTPS origin; HTTP requires STORAGE_S3_ALLOW_HTTP=true",
            ));
        }
        let endpoint = url.as_str().trim_end_matches('/').to_owned();
        let prefix = std::env::var("STORAGE_S3_PREFIX").unwrap_or_else(|_| "saturn".into());
        if !prefix.is_empty()
            && prefix.split('/').any(|part| {
                part.is_empty()
                    || part == "."
                    || part == ".."
                    || !part
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || b"-_".contains(&c))
            })
        {
            return Err(bad(
                "STORAGE_S3_PREFIX must contain slash-separated letters, numbers, hyphens or underscores",
            ));
        }
        // Import credential providers only. Storage identity and TLS policy come from
        // Saturn settings, so AWS endpoint overrides cannot mislabel saved objects.
        let mut builder = AmazonS3Builder::new();
        for name in [
            "AWS_ACCESS_KEY_ID",
            "AWS_SECRET_ACCESS_KEY",
            "AWS_SESSION_TOKEN",
            "AWS_WEB_IDENTITY_TOKEN_FILE",
            "AWS_ROLE_ARN",
            "AWS_ROLE_SESSION_NAME",
            "AWS_CONTAINER_CREDENTIALS_RELATIVE_URI",
            "AWS_CONTAINER_CREDENTIALS_FULL_URI",
            "AWS_CONTAINER_AUTHORIZATION_TOKEN_FILE",
        ] {
            if let Ok(value) = std::env::var(name)
                && !value.is_empty()
            {
                builder = builder
                    .with_config(name.to_ascii_lowercase().parse().map_err(internal)?, value);
            }
        }
        if region.is_empty() {
            return Err(bad("STORAGE_S3_REGION cannot be empty"));
        }
        let client = builder
            .with_bucket_name(&bucket)
            .with_region(&region)
            .with_endpoint(&endpoint)
            .with_virtual_hosted_style_request(false)
            .with_allow_http(allow_http)
            .with_client_options(
                ClientOptions::new()
                    .with_allow_http(allow_http)
                    .with_timeout(Duration::from_secs(120))
                    .with_connect_timeout(Duration::from_secs(10)),
            )
            .build()
            .map_err(|_| internal("Invalid S3 configuration"))?;
        Ok(Self {
            bucket,
            region,
            endpoint,
            prefix,
            client,
        })
    }
    pub async fn upload(&self, key: &str, file: &Path) -> Result<()> {
        let mut file = tokio::fs::File::open(file).await.map_err(internal)?;
        let mut upload = WriteMultipart::new(
            self.client
                .put_multipart(&key.into())
                .await
                .map_err(remote_error)?,
        );
        let mut buffer = vec![0; 1024 * 1024];
        let result: Result<()> = async {
            loop {
                upload.wait_for_capacity(2).await.map_err(remote_error)?;
                let size = file.read(&mut buffer).await.map_err(internal)?;
                if size == 0 {
                    break;
                }
                upload.write(&buffer[..size]);
            }
            Ok(())
        }
        .await;
        if let Err(error) = result {
            let _ = upload.abort().await;
            return Err(error);
        }
        upload.finish().await.map_err(remote_error)?;
        Ok(())
    }
    #[cfg(test)]
    pub async fn keys(&self) -> Vec<String> {
        use futures_util::TryStreamExt;
        use object_store::ObjectStore;
        self.client
            .list(None)
            .try_collect::<Vec<_>>()
            .await
            .unwrap()
            .into_iter()
            .map(|o| o.location.to_string())
            .collect()
    }
    pub async fn delete(&self, key: &str) -> Result<()> {
        self.client.delete(&key.into()).await.map_err(remote_error)
    }
    pub async fn signed_url(&self, key: &str) -> Result<String> {
        self.client
            .signed_url(reqwest::Method::GET, &key.into(), Duration::from_secs(300))
            .await
            .map(|u| u.to_string())
            .map_err(remote_error)
    }
    pub async fn fetch(&self, key: &str, directory: &Path) -> Result<tempfile::NamedTempFile> {
        use futures_util::TryStreamExt;
        use tokio::io::AsyncWriteExt;
        let temp = tempfile::NamedTempFile::new_in(directory).map_err(internal)?;
        let mut file = tokio::fs::File::from_std(temp.reopen().map_err(internal)?);
        let mut stream = self
            .client
            .get(&key.into())
            .await
            .map_err(remote_error)?
            .into_stream();
        while let Some(bytes) = stream.try_next().await.map_err(remote_error)? {
            file.write_all(&bytes).await.map_err(internal)?;
        }
        file.flush().await.map_err(internal)?;
        Ok(temp)
    }
}

// SDK errors can contain signed URLs or credential-bearing request details.
fn remote_error(_: object_store::Error) -> crate::ApiError {
    internal("S3 operation failed")
}
