use crate::auth::AuthConfig;
use crate::config::Config;
use crate::logger::Logger;
use anyhow::{Context, Result, anyhow};
use comfy_table::Table;
use ignore::WalkBuilder;
use inquire::{Password, Text};
use serde_json::json;
use std::io::{Cursor, Read, Write};
use zip::write::FileOptions;

/// Prompts for username/password and authenticates with the registry.
/// Stores the token in the system keyring on success.
pub async fn login() -> Result<()> {
    let username = Text::new("Username:").prompt()?;
    let username = username.trim().to_string();
    let password = Password::new("Password:")
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .without_confirmation()
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

    let status = response.status();
    let text = response.text().await?;

    if status.is_success() {
        // Parse the response and extract the token.
        let data: serde_json::Value = match serde_json::from_str(&text) {
            Ok(d) => d,
            Err(_) => {
                Logger::error(format!("Server returned invalid JSON: {}", text));
                return Err(anyhow!("Invalid server response"));
            }
        };

        let token = data["token"]
            .as_str()
            .ok_or_else(|| anyhow!("Token missing in response"))?;

        // Save credentials to disk and keyring.
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
        // Try to parse the error message from the response.
        // If it's JSON with an "error" field, use that. Otherwise use the raw text.
        let msg = match serde_json::from_str::<serde_json::Value>(&text) {
            Ok(json) => json["error"].as_str().unwrap_or(&text).to_string(),
            Err(_) => text, // Fallback for non-JSON responses (e.g. 500 errors)
        };
        Logger::error(format!("Login failed ({}): {}", status, msg));
    }

    Ok(())
}

/// Creates a new account on the registry and logs in automatically.
pub async fn signup() -> Result<()> {
    let username = Text::new("Choose Username:").prompt()?;
    let password = Password::new("Choose Password:")
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .without_confirmation() // Explicitly no confirmation—user gets one shot
        .prompt()?;

    Logger::info("Creating account on Mosaic Registry...");

    let client = reqwest::Client::new();
    let registry_url = std::env::var("MOSAIC_REGISTRY_URL")
        .unwrap_or_else(|_| "https://api.getmosaic.run".to_string());

    let response = client
        .post(format!("{}/auth/signup", registry_url))
        .json(&json!({
            "username": username,
            "password": password
        }))
        .send()
        .await?;

    let status = response.status();
    let text = response.text().await?;

    if status.is_success() {
        Logger::success(format!(
            "Account created successfully for {}!",
            Logger::highlight(&username)
        ));
        Logger::info("Logging you in automatically...");

        // Parse the response to get the auto-login token.
        let data: serde_json::Value = match serde_json::from_str(&text) {
            Ok(d) => d,
            Err(_) => {
                Logger::error(format!("Server returned invalid JSON: {}", text));
                return Err(anyhow!("Invalid server response"));
            }
        };

        let token = data["token"]
            .as_str()
            .ok_or_else(|| anyhow!("Token missing in response"))?;

        // Log them in immediately by saving the token.
        let mut auth = AuthConfig::load()?;
        auth.token = Some(token.to_string());
        auth.username = Some(username.clone());
        auth.registry_url = Some(registry_url);
        auth.save()?;

        Logger::success("Successfully logged in!");
    } else {
        let msg = match serde_json::from_str::<serde_json::Value>(&text) {
            Ok(json) => json["error"].as_str().unwrap_or(&text).to_string(),
            Err(_) => text,
        };
        Logger::error(format!("Signup failed ({}): {}", status, msg));
    }

    Ok(())
}

/// Clears all credentials from disk and keyring.
pub async fn logout() -> Result<()> {
    AuthConfig::logout()?;
    Logger::success("Logged out successfully.");
    Ok(())
}

/// Searches the registry for packages matching a query.
/// Displays results in a nice table.
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
            table.set_header(vec!["Package", "Version", "Author", "Description"]);

            for pkg in packages {
                table.add_row(vec![
                    pkg["name"].as_str().unwrap_or("unknown"),
                    pkg["version"].as_str().unwrap_or("0.0.0"),
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

/// Publishes a package to the registry.
///
/// This is the big one. Does a lot of work:
/// 1. Zips up all non-ignored files in the project
/// 2. Registers the version with the registry (creates package if needed)
/// 3. Uploads the zip blob to storage
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

    // Step 1: Create a zip file of all publishable source files
    let mut buf = Vec::new();
    {
        Logger::info("Packaging source files...");
        let mut zip = zip::ZipWriter::new(Cursor::new(&mut buf));
        let options: FileOptions<'_, ()> = FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o755);

        // Use `ignore` crate to walk files, respecting .gitignore and .mosaicignore
        let walker = WalkBuilder::new(".")
            .hidden(true) // Ignore hidden files (.git, .env, etc.)
            .add_custom_ignore_filename(".mosaicignore")
            .build();

        for result in walker {
            match result {
                Ok(entry) => {
                    let path = entry.path();
                    if path.is_dir() {
                        continue;
                    }

                    let path_str = path.to_string_lossy();

                    // Extra paranoia: manually skip common build directories even if not ignored.
                    // The `ignore` crate is usually good about this, but belt + suspenders.
                    if path_str.contains("node_modules") || path_str.contains("target") {
                        continue;
                    }

                    // Don't publish the manifest itself—that would be weird.
                    if path
                        .file_name()
                        .map(|s| s == "mosaic.toml")
                        .unwrap_or(false)
                    {
                        continue;
                    }

                    // Normalize the path for the zip file.
                    // Remove leading "./" and fix Windows path separators.
                    let name_str = if path.starts_with(".") {
                        path.strip_prefix(".")
                            .unwrap_or(path)
                            .to_string_lossy()
                            .trim_start_matches('\\')
                            .trim_start_matches('/')
                            .to_string()
                    } else {
                        path_str.to_string()
                    };

                    if name_str.is_empty() {
                        continue;
                    }

                    zip.start_file(name_str.clone(), options)?;
                    let content = std::fs::read(path)?;
                    zip.write_all(&content)?;
                }
                Err(err) => {
                    // A single file access error shouldn't kill the whole publish.
                    // Just warn and skip it.
                    Logger::warn(format!("Skipping file access error: {}", err));
                }
            }
        }
        zip.finish()?;
    }

    let client = reqwest::Client::new();

    // Step 2: Register the version with the registry.
    // If the package doesn't exist, we have to create it first.
    Logger::info("Registering version with registry...");
    let reg_res = client
        .post(format!("{}/packages/{}/versions", registry_url, name))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "version": version,
            "lua_source_url": "tbd" // Will be updated after upload
        }))
        .send()
        .await?;

    if !reg_res.status().is_success() && reg_res.status() != reqwest::StatusCode::CONFLICT {
        // 409 CONFLICT means version already exists, which is fine. Anything else is an error.
        if reg_res.status() == reqwest::StatusCode::NOT_FOUND {
            // Package doesn't exist—have to create it first before registering versions.
            Logger::info("Package doesn't exist. Creating package...");
            let create_pkg_res = client
                .post(format!("{}/packages", registry_url))
                .header("Authorization", format!("Bearer {}", token))
                .json(&json!({
                    "name": name,
                    "description": "A Mosaic package", // Placeholder, user can update later
                    "repository": "",
                    "author": auth.username.as_ref().unwrap_or(&"unknown".to_string()),
                    "created_at": 0,
                    "updated_at": 0
                }))
                .send()
                .await?;

            if !create_pkg_res.status().is_success() {
                let status = create_pkg_res.status();
                let text = create_pkg_res.text().await?;
                let msg = match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(json) => json["error"].as_str().unwrap_or(&text).to_string(),
                    Err(_) => text,
                };
                return Err(anyhow!("Failed to create package ({}): {}", status, msg));
            }

            // Now retry registering the version.
            let retry_res = client
                .post(format!("{}/packages/{}/versions", registry_url, name))
                .header("Authorization", format!("Bearer {}", token))
                .json(&json!({
                    "version": version,
                    "lua_source_url": "tbd"
                }))
                .send()
                .await?;

            if !retry_res.status().is_success()
                && retry_res.status() != reqwest::StatusCode::CONFLICT
            {
                let err: serde_json::Value = retry_res.json().await?;
                return Err(anyhow!(
                    "Failed to register version after package creation: {}",
                    err["error"]
                ));
            }
        } else {
            let err: serde_json::Value = reg_res.json().await?;
            return Err(anyhow!("Failed to register version: {}", err["error"]));
        }
    }

    // Step 3: Upload the zip blob to storage.
    // This is where the actual package code lives.
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

/// Downloads a package from the registry and extracts the first .lua file.
///
/// This is what `mosaic install` calls under the hood. Fetches the version metadata,
/// grabs the download URL, fetches the zip, and extracts the Lua source code.
pub async fn download_from_registry(name: &str, version: &str) -> Result<String> {
    let auth = AuthConfig::load()?;
    let registry_url = auth
        .registry_url
        .unwrap_or_else(|| "https://api.getmosaic.run".to_string());

    let client = reqwest::Client::new();

    // Fetch the list of versions for this package to get the download URL.
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

    // Download the zip blob from storage.
    let blob_res = client
        .get(format!("{}{}", registry_url, source_url))
        .send()
        .await?;

    let bytes = blob_res.bytes().await?;

    // Extract the first .lua file from the zip.
    // Assumes there's at least one Lua file in the package. If there's multiple,
    // we just return the first one we find. This might be a dumb assumption someday.
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
