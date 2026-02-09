use anyhow::{Result, anyhow};

pub async fn download_from_github(owner: &str, repo: &str, tag: &str) -> Result<String> {
    let url = format!(
        "https://raw.githubusercontent.com/{}/{}/{}/init.lua",
        owner, repo, tag
    );
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "mosaic-package-manager")
        .send()
        .await?;

    if !response.status().is_success() {
        // Fallback to searching for other common filenames if init.lua doesn't exist?
        // For now, let's assume init.lua is the entry point.
        return Err(anyhow!(
            "Failed to download package from {}: {}",
            url,
            response.status()
        ));
    }

    Ok(response.text().await?)
}
