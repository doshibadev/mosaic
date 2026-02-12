use clap::{Parser, Subcommand};

/// Main CLI entry point. Pretty straightforward—parse args and dispatch to subcommands.
/// The `#[command(subcommand)]` macro does most of the heavy lifting for us.
#[derive(Parser)]
#[command(name = "mosaic")]
#[command(about = "Polytoria Package Manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Let users override the registry URL if they're running their own instance.
    /// Useful for testing or if someone wants to self-host the registry.
    /// `global = true` means it works with any subcommand.
    #[arg(long, global = true)]
    pub api_url: Option<String>,

    /// Enable verbose logging for debugging.
    /// Prints detailed error messages and other internal info.
    #[arg(long, short, global = true)]
    pub verbose: bool,
}

/// Every command the CLI supports. Pretty much what you'd expect from a package manager.
#[derive(Subcommand)]
pub enum Commands {
    /// Sets up mosaic.toml in the current directory.
    /// Nothing fancy—just scaffolds the config file.
    Init,

    /// Install a package. Can handle:
    /// - Registry packages: `logger@1.0.0`
    /// - GitHub repos: `github:username/repo` (might add this someday)
    Install {
        /// Package name (e.g. logger@1.0.0 or github:user/repo)
        package: Option<String>,
    },

    /// Removes a package from mosaic.toml and from your .poly file.
    /// Just deletes the dependency, nothing complicated.
    Remove {
        /// Package name to remove
        package: String,
    },

    /// Lists everything installed. Reads from mosaic.toml.
    /// Useful if you forget what you added.
    List,

    /// Updates all packages to their latest versions.
    /// Respects version constraints (if we implement those someday).
    Update,

    /// Logs you in. Stores credentials securely (hopefully).
    /// Prompts for username/password and stashes the token in the system keyring.
    Login,

    /// Removes your stored credentials everywhere.
    /// Keyring + config file. You're fully logged out after this.
    Logout,

    /// Creates a new account on the registry.
    /// Just a convenience wrapper around the API endpoint.
    Signup,

    /// Publishes your package to the registry.
    /// Reads from mosaic.toml unless you override the version.
    Publish {
        /// Optional version string (defaults to mosaic.toml version)
        /// Useful if you want to bump the version from the CLI instead of editing the file.
        version: Option<String>,
    },

    /// Searches the registry for packages.
    /// Pretty basic—just a text query. Fuzzy matching would be nice but... someday.
    Search {
        /// Search query
        query: String,
    },

    /// Shows details about a package without installing it.
    /// Author, description, version... the usual stuff.
    /// Basically `npm view` but less verbose.
    Info {
        /// Package name to look up
        package: String,
    },

    /// Removes a version from the registry.
    /// Only works within 24 hours of publishing and if no one else depends on it.
    /// Use this if you accidentally uploaded your cat's photos instead of code.
    Unpublish {
        /// Package name and version (e.g. logger@1.0.0)
        package: String,
    },
}
