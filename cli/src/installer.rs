use crate::logger::Logger;
use crate::registry;
use crate::xml_handler;
use anyhow::{Result, anyhow};
use comfy_table::Table;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;

pub async fn install_package(package_query: &str) -> Result<String> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("Resolving {}", Logger::highlight(package_query)));
    pb.enable_steady_tick(std::time::Duration::from_millis(120));

    // Registry format: name@version
    let parts: Vec<&str> = package_query.split('@').collect();
    if parts.len() != 2 {
        pb.finish_and_clear();
        return Err(anyhow!("Invalid package format. Expected: name@version"));
    }
    let name = parts[0];
    let version = parts[1];

    pb.set_message(format!(
        "Downloading {} from Registry...",
        Logger::highlight(name)
    ));

    let lua_code = registry::download_from_registry(name, version).await?;

    pb.set_message("Locating .poly project...");
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

    pb.set_message(format!(
        "Injecting {} into project",
        Logger::highlight(name)
    ));
    let poly_content = fs::read_to_string(&poly_path)?;
    let new_content = xml_handler::inject_module_script(&poly_content, name, &lua_code)?;

    fs::write(&poly_path, new_content)?;
    pb.finish_and_clear();

    Logger::success(format!(
        "Installed {} into {}",
        Logger::brand_text(name),
        Logger::highlight(poly_path.to_string_lossy())
    ));

    Ok(name.to_string())
}

pub async fn install_all() -> Result<()> {
    let config = crate::config::Config::load()?;
    Logger::header(format!(
        "Installing dependencies for {}",
        config.package.name
    ));

    for (name, query) in &config.dependencies {
        Logger::command("mosaic", format!("Processing {} ({})", name, query));
        install_package(query).await?;
    }

    Logger::success("All dependencies are up to date!");
    Ok(())
}

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

pub async fn update_all() -> Result<()> {
    Logger::info("Syncing project dependencies...");
    install_all().await?;
    Ok(())
}

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

    config.remove_dependency(name);
    config.save()?;

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
        pb.finish_and_clear();
        Logger::success(format!(
            "Removed {} from mosaic.toml",
            Logger::highlight(name)
        ));
    }

    Ok(())
}
