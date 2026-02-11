use anyhow::{Context, Result};
use directories::ProjectDirs;
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    #[serde(skip)]
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
        let mut config: AuthConfig = if path.exists() {
            let content = fs::read_to_string(path)?;
            toml::from_str(&content)?
        } else {
            Self::default()
        };

        // Load token from keyring if username is present
        if let Some(raw_username) = &config.username {
            let username = raw_username.trim();
            // Use new_with_target to be explicit about the target store on Windows
            if let Ok(entry) = Entry::new_with_target("mosaic-package-manager", "mosaic-package-manager", username) {
                if let Ok(token) = entry.get_password() {
                    config.token = Some(token);
                }
            }
        }

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::get_path()?;
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;

        // Save token to keyring
        if let Some(raw_username) = &self.username {
             let username = raw_username.trim();
             if let Ok(entry) = Entry::new_with_target("mosaic-package-manager", "mosaic-package-manager", username) {
                if let Some(token) = &self.token {
                    let _ = entry.set_password(token);
                } else {
                    let _ = entry.delete_credential();
                }
             }
        }

        Ok(())
    }

    pub fn logout() -> Result<()> {
        let path = Self::get_path()?;
        
        // 1. Try to load config to get username for keyring cleanup
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            if let Ok(config) = toml::from_str::<AuthConfig>(&content) {
                if let Some(raw_username) = config.username {
                    let username = raw_username.trim();
                    if let Ok(entry) = Entry::new_with_target("mosaic-package-manager", "mosaic-package-manager", username) {
                        let _ = entry.delete_credential();
                    }
                }
            }
            // 2. Delete the file
            fs::remove_file(path)?;
        }
        
        Ok(())
    }
}
