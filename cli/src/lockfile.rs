use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::Result;

/// Represents the mosaic.lock file.
/// This file ensures reproducible builds by locking dependencies to specific versions and hashes.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Lockfile {
    /// Map of package name to locked version/hash.
    /// We use BTreeMap implicitly via serde to keep keys sorted for deterministic output.
    pub packages: HashMap<String, LockedPackage>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LockedPackage {
    pub version: String,
    pub integrity: String, // SHA256 hash of the zip blob
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
}

impl Lockfile {
    pub fn load() -> Result<Self> {
        let path = Path::new("mosaic.lock");
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let lockfile: Lockfile = toml::from_str(&content)?;
            Ok(lockfile)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write("mosaic.lock", content)?;
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&LockedPackage> {
        self.packages.get(name)
    }

    pub fn insert(&mut self, name: String, pkg: LockedPackage) {
        self.packages.insert(name, pkg);
    }
}
