use crate::auth::AuthConfig;
use crate::config::Config;
use crate::logger::Logger;
use anyhow::{Context, Result, anyhow};
use comfy_table::Table;
use inquire::{Password, Text};
use serde_json::json;
use std::io::{Cursor, Read, Write};
use walkdir::WalkDir;
use zip::write::FileOptions;

pub async fn login() -> Result<()> {
    let username = Text::new("Username:").prompt()?;
    let password = Password::new("Password:")
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .prompt()?;

    Logger::info("Authenticating with registry...");

    let client = reqwest::Client::new();
    let registry_url = std::env::var("MOSAIC_REGISTRY_URL")
        .unwrap_or_else(|_| "https://api.getmosaic.run".to_string());

    let response = client
        .post(format!("{}/auth/login", registry_url))
        .json(&json!({
            "username": username,
            "password": password
        }))
        .send()
        .await?;

    if response.status().is_success() {
        let data: serde_json::Value = response.json().await?;
        let token = data["token"]
            .as_str()
            .ok_or_else(|| anyhow!("Token missing in response"))?;

        let mut auth = AuthConfig::load()?;
        auth.token = Some(token.to_string());
        auth.username = Some(username.clone());
        auth.registry_url = Some(registry_url);
        auth.save()?;

        Logger::success(format!(
            "Successfully logged in as {}!",
            Logger::highlight(&username)
        ));
    } else {
        let error: serde_json::Value = response.json().await?;
        let msg = error["error"].as_str().unwrap_or("Unknown error");
        Logger::error(format!("Login failed: {}", msg));
    }

    Ok(())
}

pub async fn search(query: String) -> Result<()> {
    let auth = AuthConfig::load()?;
    let registry_url = auth
        .registry_url
        .unwrap_or_else(|| "https://api.getmosaic.run".to_string());

    Logger::info(format!(
        "Searching registry for {}...",
        Logger::highlight(&query)
    ));

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/packages/search", registry_url))
        .query(&[("q", &query)])
        .send()
        .await?;

    if response.status().is_success() {
        let packages: Vec<serde_json::Value> = response.json().await?;
        if packages.is_empty() {
            Logger::error("No packages found.");
        } else {
            let mut table = Table::new();
            table.set_header(vec!["Package", "Author", "Description"]);

            for pkg in packages {
                table.add_row(vec![
                    pkg["name"].as_str().unwrap_or("unknown"),
                    pkg["author"].as_str().unwrap_or("unknown"),
                    pkg["description"].as_str().unwrap_or("No description"),
                ]);
            }
            println!("\n{}", table);
        }
    } else {
        Logger::error("Search failed.");
    }

    Ok(())
}

pub async fn publish(version_override: Option<&str>) -> Result<()> {
    let auth = AuthConfig::load()?;
    let token = auth
        .token
        .as_ref()
        .context("Not logged in. Run 'mosaic login' first.")?;
    let registry_url = auth
        .registry_url
        .as_ref()
        .context("Registry URL missing in config.")?;

    let config = Config::load().context("Could not find mosaic.toml in current directory.")?;
    let name = &config.package.name;
    let version = version_override.unwrap_or(&config.package.version);

    Logger::command("publish", format!("{}@{}", name, version));

    // 1. Create Zip
    let mut buf = Vec::new();
    {
        Logger::info("Packaging source files...");
        let mut zip = zip::ZipWriter::new(Cursor::new(&mut buf));
        let options: FileOptions<'_, ()> = FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o755);

        for entry in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let relative_path = path.strip_prefix("./").unwrap_or(path);

            if path.is_file() {
                let name_str = relative_path.to_string_lossy();
                if name_str.starts_with(".")
                    || name_str.contains("node_modules")
                    || name_str.contains("target")
                    || name_str == "mosaic.toml"
                {
                    continue;
                }

                zip.start_file(name_str.to_string(), options)?;
                let content = std::fs::read(path)?;
                zip.write_all(&content)?;
            }
        }
        zip.finish()?;
    }

    let client = reqwest::Client::new();

    // 2. Register Version (if it doesn't exist)
    Logger::info("Registering version with registry...");
    let reg_res = client
        .post(format!("{}/packages/{}/versions", registry_url, name))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "version": version,
            "lua_source_url": "tbd"
        }))
        .send()
        .await?;

    if !reg_res.status().is_success() && reg_res.status() != reqwest::StatusCode::CONFLICT {
        let err: serde_json::Value = reg_res.json().await?;
        return Err(anyhow!("Failed to register version: {}", err["error"]));
    }

    // 3. Upload Blob
    Logger::info("Uploading package blob to storage...");
    let upload_res = client
        .post(format!(
            "{}/packages/{}/versions/{}/upload",
            registry_url, name, version
        ))
        .header("Authorization", format!("Bearer {}", token))
        .body(buf)
        .send()
        .await?;

    if upload_res.status().is_success() {
        Logger::success(format!(
            "Successfully published {}@{}!",
            Logger::highlight(name),
            Logger::brand_text(version)
        ));
    } else {
        let err: serde_json::Value = upload_res.json().await?;
        Logger::error(format!("Publish failed: {}", err["error"]));
    }

    Ok(())
}

pub async fn download_from_registry(name: &str, version: &str) -> Result<String> {
    let auth = AuthConfig::load()?;
    let registry_url = auth
        .registry_url
        .unwrap_or_else(|| "https://api.getmosaic.run".to_string());

    let client = reqwest::Client::new();

    let versions_res = client
        .get(format!("{}/packages/{}/versions", registry_url, name))
        .send()
        .await?;

    let versions: Vec<serde_json::Value> = versions_res.json().await?;
    let target_version = versions
        .into_iter()
        .find(|v| v["version"].as_str() == Some(version))
        .ok_or_else(|| anyhow!("Version {} not found for package {}", version, name))?;

    let source_url = target_version["lua_source_url"]
        .as_str()
        .ok_or_else(|| anyhow!("Source URL missing for package {}@{}", name, version))?;

    let blob_res = client
        .get(format!("{}{}", registry_url, source_url))
        .send()
        .await?;

    let bytes = blob_res.bytes().await?;

    let reader = Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(reader)?;

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        if file.name().ends_with(".lua") {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            return Ok(content);
        }
    }

    Err(anyhow!("No .lua file found in package zip"))
}
