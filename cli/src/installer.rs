use crate::github;
use crate::xml_handler;
use anyhow::{Result, anyhow};
use std::fs;

pub async fn install_package(package_query: &str) -> Result<String> {
    // Basic parser for github:owner/repo@version
    if !package_query.starts_with("github:") {
        return Err(anyhow!(
            "Only github packages are supported in MVP: github:owner/repo@version"
        ));
    }

    let parts: Vec<&str> = package_query
        .trim_start_matches("github:")
        .split('@')
        .collect();
    if parts.len() != 2 {
        return Err(anyhow!(
            "Invalid package format. Expected: github:owner/repo@version"
        ));
    }

    let repo_parts: Vec<&str> = parts[0].split('/').collect();
    if repo_parts.len() != 2 {
        return Err(anyhow!("Invalid repo format. Expected: owner/repo"));
    }

    let owner = repo_parts[0];
    let repo_name = repo_parts[1];
    let version = parts[1];

    println!("Downloading {}/{} @ {}...", owner, repo_name, version);
    let lua_code = github::download_from_github(owner, repo_name, version).await?;

    // Find the first .poly file in the current directory
    let entries = fs::read_dir(".")?;
    let mut poly_file_path = None;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("poly") {
            poly_file_path = Some(path);
            break;
        }
    }

    let poly_path =
        poly_file_path.ok_or_else(|| anyhow!("No .poly file found in the current directory"))?;
    println!("Found project file: {:?}", poly_path);

    let poly_content = fs::read_to_string(&poly_path)?;
    let new_content = xml_handler::inject_module_script(&poly_content, repo_name, &lua_code)?;

    fs::write(&poly_path, new_content)?;
    println!("Successfully installed {} into {:?}", repo_name, poly_path);

    Ok(repo_name.to_string())
}

pub async fn install_all() -> Result<()> {
    let config = crate::config::Config::load()?;
    println!("Installing all dependencies for {}...", config.package.name);

    for (name, query) in &config.dependencies {
        println!("Installing dependency: {} ({})", name, query);
        install_package(query).await?;
    }

    Ok(())
}
