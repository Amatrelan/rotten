use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
    #[arg(short, long, default_value_t = 3, action = clap::ArgAction::Count)]
    pub verbosity: u8,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Initialize rotten folder
    Init {
        /// If overwrite is given then this will generate new rotten.toml
        #[arg(short, long)]
        overwrite: bool,
    },
    /// Setups state file, run in folder where your rotten.toml is
    ///
    /// Setup will overwrite state file always
    Setup,
    /// Link all rotten managed things into places
    Link {
        #[arg(short, long)]
        overwrite: bool,
        #[arg(short, long)]
        profiles: Vec<String>,
    },
    /// Unlink all existing symlinks (not working yet)
    Unlink,
}
