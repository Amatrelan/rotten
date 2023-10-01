use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Initialize rotten folder
    ///
    /// Also setups `rot.toml`
    Init,
    /// Create new rotten folder and setup, also setups `rot.toml`
    ///
    /// Also setups `rot.toml`.
    New { path: String },
    /// Add new path to rotten, this will copy/move `source` path to `target` path and automatically
    /// symlink it back to `source` path
    Add {
        // Provide custom name
        #[clap(short, long)]
        name: Option<String>,

        /// Source path to add rotten managed folder.
        #[clap(short, long, value_name = "FILE")]
        source: String,
        /// Target path where in rotten folder this is added.
        #[clap(short, long, value_name = "FILE")]
        target: String,
    },
    /// Link all rotten managed things into places
    Link,
}
