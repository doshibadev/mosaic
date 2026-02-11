use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The main config struct that mirrors mosaic.toml.
/// Split into package metadata and dependencies because it's cleaner that way.
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub package: PackageConfig,
    pub dependencies: HashMap<String, String>,
}

impl Config {
    /// Creates a default config for a new project.
    /// Starts with version 0.1.0 because that's what everyone does anyway.
    pub fn default(name: &str) -> Self {
        Self {
            package: PackageConfig {
                name: name.to_string(),
                version: "0.1.0".to_string(),
            },
            dependencies: HashMap::new(),
        }
    }

    /// Reads mosaic.toml from disk and parses it.
    /// Assumes you're running from the project root. Will fail if you're not.
    pub fn load() -> anyhow::Result<Self> {
        let content = std::fs::read_to_string("mosaic.toml")?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Adds or updates a dependency in memory.
    /// Doesn't write to disk—call save() when you're ready.
    /// The query is usually something like "1.0.0" or "^1.2.0" but we don't validate it here.
    pub fn add_dependency(&mut self, name: &str, query: &str) {
        self.dependencies
            .insert(name.to_string(), query.to_string());
    }

    /// Removes a dependency from the config.
    /// Again, in-memory only. You have to save() to persist it.
    pub fn remove_dependency(&mut self, name: &str) {
        self.dependencies.remove(name);
    }

    /// Writes the config back to mosaic.toml.
    /// Uses pretty TOML formatting so it's actually readable (learned that lesson early).
    pub fn save(&self) -> anyhow::Result<()> {
        let toml = toml::to_string_pretty(self)?;
        std::fs::write("mosaic.toml", toml)?;
        Ok(())
    }
}

/// Package metadata—just name and version.
/// Could expand this later if we need more fields (author, license, etc).
/// Right now it's kept simple because YAGNI.
#[derive(Serialize, Deserialize, Debug)]
pub struct PackageConfig {
    pub name: String,
    pub version: String,
}
