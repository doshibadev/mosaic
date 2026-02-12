use aws_config::BehaviorVersion;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::Region;
use aws_sdk_s3::primitives::ByteStream;
use std::env;

/// Wrapper around AWS S3/R2 for storing package blobs.
///
/// Cloudflare R2 is S3-compatible, so we use the AWS SDK directly.
/// All package zips get uploaded here by hash, then we store the R2 URL in the database.
#[derive(Clone)]
pub struct StorageService {
    client: Client,
    bucket: String,
}

impl StorageService {
    /// Initializes the S3 client with R2 credentials.
    ///
    /// Reads from environment variables:
    /// - R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY (API credentials)
    /// - R2_ENDPOINT (R2-specific S3 endpoint, e.g., https://xxx.r2.cloudflarestorage.com)
    /// - R2_BUCKET_NAME (defaults to "mosaic-packages" if not set)
    ///
    /// R2 uses "auto" region and custom endpoint URL instead of traditional AWS regions.
    pub async fn new() -> Self {
        let access_key = env::var("R2_ACCESS_KEY_ID").expect("R2_ACCESS_KEY_ID must be set");
        let secret_key =
            env::var("R2_SECRET_ACCESS_KEY").expect("R2_SECRET_ACCESS_KEY must be set");
        let endpoint = env::var("R2_ENDPOINT").expect("R2_ENDPOINT must be set");
        let bucket = env::var("R2_BUCKET_NAME").unwrap_or_else(|_| "mosaic-packages".to_string());

        // Create static credentials (not using STS or temporary credentials).
        // R2 doesn't really care about regions, but the SDK requires one, so we use "auto".
        let credentials =
            aws_sdk_s3::config::Credentials::new(access_key, secret_key, None, None, "Static");

        let region_provider = RegionProviderChain::default_provider().or_else(Region::new("auto"));

        // Build the AWS config but override the endpoint to point at R2 instead of AWS S3.
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .credentials_provider(credentials)
            .endpoint_url(endpoint)
            .load()
            .await;

        let client = Client::new(&config);

        Self { client, bucket }
    }

    /// Uploads a package blob to R2.
    ///
    /// Uses the content hash as the S3 key so we never store duplicates.
    /// If the same blob is uploaded twice, it just overwrites (which is fine).
    pub async fn upload_blob(&self, hash: &str, data: Vec<u8>) -> anyhow::Result<()> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(format!("blobs/{}", hash))
            .body(ByteStream::from(data))
            .content_type("application/octet-stream")
            .send()
            .await?;
        Ok(())
    }

    /// Downloads a package blob from R2 by hash.
    pub async fn get_blob(&self, hash: &str) -> anyhow::Result<Vec<u8>> {
        let output = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(format!("blobs/{}", hash))
            .send()
            .await?;

        // The body is a stream, so we have to collect it into bytes.
        let data = output.body.collect().await?.into_bytes();
        Ok(data.to_vec())
    }

    /// Deletes a package blob from R2.
    /// Used for rolling back failed uploads.
    pub async fn delete_blob(&self, hash: &str) -> anyhow::Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(format!("blobs/{}", hash))
            .send()
            .await?;
        Ok(())
    }
}
