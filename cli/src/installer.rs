use crate::logger::Logger;
use crate::registry;
use crate::xml_handler;
use anyhow::{Result, anyhow};
use comfy_table::Table;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::{HashMap, HashSet};
use std::fs;

/// Installs a package into the current project, including all its dependencies.
///
/// Handles both explicit versions (name@version) and latest-version lookup.
/// Returns the resolved (name, version) tuple so main.rs can update mosaic.toml.
pub async fn install_package(package_query: &str) -> Result<(String, String)> {
    let mut visited = HashSet::new();
    let mut recursion_stack = Vec::new();

    // We start the recursive process.
    // The first package is the one the user explicitly requested.
    let (name, version) = resolve_and_install(package_query, &mut visited, &mut recursion_stack).await?;

    Ok((name, version))
}

/// The recursive engine behind install_package.
///
/// 1. Resolves version
/// 2. Checks for circular dependencies (DFS)
/// 3. Installs dependencies first
/// 4. Injects the package itself
async fn resolve_and_install(
    package_query: &str,
    visited: &mut HashSet<String>,
    recursion_stack: &mut Vec<String>,
) -> Result<(String, String)> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("Resolving {}", Logger::highlight(package_query)));
    pb.enable_steady_tick(std::time::Duration::from_millis(120));

    // 1. Resolve Name & Version
    let (name, version) = if package_query.contains('@') {
        let parts: Vec<&str> = package_query.split('@').collect();
        if parts.len() != 2 {
            pb.finish_and_clear();
            return Err(anyhow!(
                "Invalid package format. Expected: name or name@version"
            ));
        }
        (parts[0].to_string(), parts[1].to_string())
    } else {
        pb.set_message(format!(
            "Fetching latest version for {}...",
            Logger::highlight(package_query)
        ));
        let registry_url = std::env::var("MOSAIC_REGISTRY_URL")
            .unwrap_or_else(|_| "https://api.getmosaic.run".to_string());

        let client = reqwest::Client::new();
        let res = client
            .get(format!("{}/packages/{}", registry_url, package_query))
            .send()
            .await?;

        if !res.status().is_success() {
            pb.finish_and_clear();
            return Err(anyhow!("Package not found in registry: {}", package_query));
        }

        let pkg: serde_json::Value = res.json().await?;
        let latest_version = pkg["version"]
            .as_str()
            .ok_or_else(|| anyhow!("Could not determine latest version"))?
            .to_string();

        (package_query.to_string(), latest_version)
    };

    // 2. Circular Dependency Check (DFS)
    // If we're already installing this package in the current branch of the tree, it's a cycle.
    if recursion_stack.contains(&name) {
        pb.finish_and_clear();
        let mut cycle = recursion_stack.join(" -> ");
        cycle.push_str(&format!(" -> {}", name));
        return Err(anyhow!("Circular dependency detected: {}", cycle));
    }

    // 3. Skip if already visited
    // No need to install the same package twice if multiple dependencies point to it.
    if visited.contains(&name) {
        pb.finish_and_clear();
        return Ok((name, version));
    }

    // Mark as currently visiting
    recursion_stack.push(name.clone());

    // 4. Fetch Metadata & Dependencies
    // We need to know what this package depends on BEFORE we download the blob.
    let registry_url = std::env::var("MOSAIC_REGISTRY_URL")
        .unwrap_or_else(|_| "https://api.getmosaic.run".to_string());
    
    let client = reqwest::Client::new();
    let res = client
        .get(format!("{}/packages/{}/versions", registry_url, name))
        .send()
        .await?;

    let versions: Vec<serde_json::Value> = res.json().await?;
    let version_meta = versions
        .into_iter()
        .find(|v| v["version"].as_str() == Some(&version))
        .ok_or_else(|| anyhow!("Version {} not found for {}", version, name))?;

    // Extract dependencies if any
    if let Some(deps) = version_meta["dependencies"].as_object() {
        if !deps.is_empty() {
            pb.set_message(format!("Installing dependencies for {}...", name));
            for (dep_name, dep_version) in deps {
                let dep_query = format!("{}@{}", dep_name, dep_version.as_str().unwrap_or("*"));
                // Recursively call ourselves. This builds the tree bottom-up.
                Box::pin(resolve_and_install(&dep_query, visited, recursion_stack)).await?;
            }
        }
    }

    // 5. Download & Inject
    pb.set_message(format!(
        "Downloading {}@{}...",
        Logger::highlight(&name),
        Logger::brand_text(&version)
    ));

    let lua_code = registry::download_from_registry(&name, &version).await?;

    // Find the .poly file.
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

    let poly_path = match poly_file_path {
        Some(path) => path,
        None => {
            pb.finish_and_clear();
            return Err(anyhow!("No .poly file found in the current directory"));
        }
    };

    pb.set_message(format!("Injecting {} into project...", name));
    let poly_content = fs::read_to_string(&poly_path)?;
    let new_content = xml_handler::inject_module_script(&poly_content, &name, &lua_code)?;

    fs::write(&poly_path, new_content)?;
    
    // Done with this branch
    visited.insert(name.clone());
    recursion_stack.pop();
    
    pb.finish_and_clear();
    Logger::success(format!(
        "Installed {}@{} into {}",
        Logger::brand_text(&name),
        Logger::brand_text(&version),
        Logger::highlight(poly_path.to_string_lossy())
    ));

    Ok((name, version))
}

/// Installs everything listed in mosaic.toml.
/// Useful for CI/CD or when you just cloned a project and need everything.
pub async fn install_all() -> Result<()> {
    let config = crate::config::Config::load()?;
    Logger::header(format!(
        "Installing dependencies for {}",
        config.package.name
    ));

    for (name, query) in &config.dependencies {
        Logger::command("mosaic", format!("Processing {} ({})", name, query));
        // Just install what's already in the config. No need to update anything.
        // query is usually a version constraint like "1.0.0" or "^1.2.0"
        let _ = install_package(&format!("{}@{}", name, query)).await?;
    }

    Logger::success("All dependencies are up to date!");
    Ok(())
}

/// Prints the project config and list of installed packages in a nice table.
/// Mostly for humans to read—not really for parsing.
pub async fn list_packages() -> Result<()> {
    let config = crate::config::Config::load()?;

    Logger::header("Project Environment");
    println!("{} {}", Logger::brand_text("Name:"), config.package.name);
    println!(
        "{} v{}",
        Logger::brand_text("Version:"),
        config.package.version
    );

    if config.dependencies.is_empty() {
        Logger::info("No dependencies installed.");
        return Ok(());
    }

    Logger::header("Dependencies");
    let mut table = Table::new();
    table.set_header(vec!["Package", "Source/Query"]);

    for (name, query) in &config.dependencies {
        table.add_row(vec![name.to_string(), query.to_string()]);
    }

    println!("{}", table);
    Ok(())
}

/// Syncs all dependencies by re-installing everything.
/// Basically a wrapper around install_all() with slightly better messaging.
pub async fn update_all() -> Result<()> {
    Logger::info("Syncing project dependencies...");
    install_all().await?;
    Ok(())
}

/// Removes a package from mosaic.toml and the .poly file.
/// Does the work in two places because they need to stay in sync.
pub async fn remove_package(name: &str) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.red} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(120));
    pb.set_message(format!("Removing {}...", name));

    let mut config = crate::config::Config::load()?;
    if !config.dependencies.contains_key(name) {
        pb.finish_and_clear();
        Logger::error(format!("Package {} not found in mosaic.toml", name));
        return Ok(());
    }

    // Remove from the config first.
    config.remove_dependency(name);
    config.save()?;

    // Now find the .poly file and remove it from there too.
    // If the .poly file doesn't exist, that's weird but not a hard error—
    // the main thing is the config is cleaned up.
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

    if let Some(poly_path) = poly_file_path {
        let poly_content = fs::read_to_string(&poly_path)?;
        let new_content = xml_handler::remove_module_script(&poly_content, name)?;
        fs::write(&poly_path, new_content)?;
        pb.finish_and_clear();
        Logger::success(format!(
            "Removed {} from mosaic.toml and {}",
            Logger::highlight(name),
            Logger::highlight(poly_path.to_string_lossy())
        ));
    } else {
        // .poly file doesn't exist, but we already updated the config so we're good.
        pb.finish_and_clear();
        Logger::success(format!(
            "Removed {} from mosaic.toml",
            Logger::highlight(name)
        ));
    }

    Ok(())
}