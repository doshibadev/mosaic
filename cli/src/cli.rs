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
}
