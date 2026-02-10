use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mosaic")]
#[command(about = "Polytoria Package Manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new mosaic project
    Init,
    /// Install a package
    Install {
        /// Package name (e.g. logger@1.0.0 or github:user/repo)
        package: Option<String>,
    },
    /// Remove a package
    Remove {
        /// Package name to remove
        package: String,
    },
    /// List all installed packages
    List,
    /// Update all installed packages
    Update,
    /// Login to the Mosaic Registry
    Login,
    /// Publish a package to the Mosaic Registry
    Publish {
        /// Optional version string (defaults to mosaic.toml version)
        version: Option<String>,
    },
    /// Search for packages in the Mosaic Registry
    Search {
        /// Search query
        query: String,
    },
}
