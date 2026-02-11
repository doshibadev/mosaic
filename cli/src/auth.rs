use anyhow::{Context, Result};
use directories::ProjectDirs;
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Auth config split across two storage systems because I didn't want tokens in plaintext files.
/// username + registry_url live in TOML on disk. Token lives in the system keyring (if you're lucky).
/// #[serde(skip)] makes sure the token never gets serialized—learned that the hard way.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    #[serde(skip)]
    pub token: Option<String>,
    pub username: Option<String>,
    pub registry_url: Option<String>,
}

impl AuthConfig {
    /// Gets the config directory using ProjectDirs to respect OS conventions.
    /// Creates the directory if it doesn't exist. Will panic if your OS is from the 90s.
    pub fn get_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "mosaic", "mosaic")
            .context("Could not determine config directory")?;
        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir)?;
        Ok(config_dir.join("auth.toml"))
    }

    /// Loads config from disk + tries to pull the token from the system keyring.
    ///
    /// If the keyring is locked/broken/whatever, we just continue without a token.
    /// The user will get a proper "not authenticated" error later if they actually need it.
    /// This is intentional—don't make startup fail because some system daemon is being weird.
    pub fn load() -> Result<Self> {
        let path = Self::get_path()?;
        let mut config: AuthConfig = if path.exists() {
            let content = fs::read_to_string(path)?;
            toml::from_str(&content)?
        } else {
            Self::default()
        };

        if let Some(raw_username) = &config.username {
            let username = raw_username.trim();
            // new_with_target here because Windows Credential Manager is... special.
            // Without being explicit, keyring lookups fail silently on some systems.
            // Yes, I've debugged this at 2am.
            if let Ok(entry) =
                Entry::new_with_target("mosaic-package-manager", "mosaic-package-manager", username)
            {
                if let Ok(token) = entry.get_password() {
                    config.token = Some(token);
                }
            }
        }

        Ok(config)
    }

    /// Writes config to disk + syncs the token to the system keyring.
    /// Keeps everything in sync because the previous maintainer learned this lesson the hard way.
    pub fn save(&self) -> Result<()> {
        let path = Self::get_path()?;
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;

        if let Some(raw_username) = &self.username {
            let username = raw_username.trim();
            if let Ok(entry) =
                Entry::new_with_target("mosaic-package-manager", "mosaic-package-manager", username)
            {
                if let Some(token) = &self.token {
                    let _ = entry.set_password(token);
                } else {
                    // Token is gone, so delete the keyring entry.
                    // Don't want some old token hanging around if the user logs out.
                    let _ = entry.delete_credential();
                }
            }
        }

        Ok(())
    }

    /// Nukes all auth data everywhere: config file + keyring.
    ///
    /// Has to load the config first just to get the username for keyring cleanup.
    /// It's dumb, but it works. Keyring failures are silently ignored because
    /// the real important part is deleting the config file anyway.
    pub fn logout() -> Result<()> {
        let path = Self::get_path()?;

        if path.exists() {
            let content = fs::read_to_string(&path)?;
            if let Ok(config) = toml::from_str::<AuthConfig>(&content) {
                if let Some(raw_username) = config.username {
                    let username = raw_username.trim();
                    if let Ok(entry) = Entry::new_with_target(
                        "mosaic-package-manager",
                        "mosaic-package-manager",
                        username,
                    ) {
                        let _ = entry.delete_credential();
                    }
                }
            }
            fs::remove_file(path)?;
        }

        Ok(())
    }
}
