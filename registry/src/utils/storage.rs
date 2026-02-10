use aws_config::BehaviorVersion;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::Region;
use aws_sdk_s3::primitives::ByteStream;
use std::env;

#[derive(Clone)]
pub struct StorageService {
    client: Client,
    bucket: String,
}

impl StorageService {
    pub async fn new() -> Self {
        let access_key = env::var("R2_ACCESS_KEY_ID").expect("R2_ACCESS_KEY_ID must be set");
        let secret_key =
            env::var("R2_SECRET_ACCESS_KEY").expect("R2_SECRET_ACCESS_KEY must be set");
        let endpoint = env::var("R2_ENDPOINT").expect("R2_ENDPOINT must be set");
        let bucket = env::var("R2_BUCKET_NAME").unwrap_or_else(|_| "mosaic-packages".to_string());

        let credentials =
            aws_sdk_s3::config::Credentials::new(access_key, secret_key, None, None, "Static");

        let region_provider = RegionProviderChain::default_provider().or_else(Region::new("auto"));

        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .credentials_provider(credentials)
            .endpoint_url(endpoint)
            .load()
            .await;

        let client = Client::new(&config);

        Self { client, bucket }
    }

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

    pub async fn get_blob(&self, hash: &str) -> anyhow::Result<Vec<u8>> {
        let output = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(format!("blobs/{}", hash))
            .send()
            .await?;

        let data = output.body.collect().await?.into_bytes();
        Ok(data.to_vec())
    }
}
