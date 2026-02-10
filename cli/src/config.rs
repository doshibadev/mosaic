use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub package: PackageConfig,
    pub dependencies: HashMap<String, String>,
}

impl Config {
    pub fn default(name: &str) -> Self {
        Self {
            package: PackageConfig {
                name: name.to_string(),
                version: "0.1.0".to_string(),
            },
            dependencies: HashMap::new(),
        }
    }

    pub fn load() -> anyhow::Result<Self> {
        let content = std::fs::read_to_string("mosaic.toml")?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn add_dependency(&mut self, name: &str, query: &str) {
        self.dependencies
            .insert(name.to_string(), query.to_string());
    }

    pub fn remove_dependency(&mut self, name: &str) {
        self.dependencies.remove(name);
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let toml = toml::to_string_pretty(self)?;
        std::fs::write("mosaic.toml", toml)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageConfig {
    pub name: String,
    pub version: String,
}
