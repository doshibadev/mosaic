use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    pub token: Option<String>,
    pub username: Option<String>,
    pub registry_url: Option<String>,
}

impl AuthConfig {
    pub fn get_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "mosaic", "mosaic")
            .context("Could not determine config directory")?;
        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir)?;
        Ok(config_dir.join("auth.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::get_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::get_path()?;
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
