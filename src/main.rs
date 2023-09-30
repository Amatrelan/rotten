use std::path::PathBuf;

/// Simple tool to symlink dotfiles
///
/// # TODO list
/// - add md5 checksumming and validate files after.
use clap::Parser;

mod cli;
mod config;
mod utils;

fn main() -> anyhow::Result<()> {
    let matches = cli::CLI::parse();

    match matches.command {
        cli::Commands::Init => {
            let path = std::env::current_dir()?;
            config::initialize_config(&path)?;
            println!("Initialized rotten to {path:?}");
            Ok(())
        }
        cli::Commands::New { path } => {
            let path = utils::parse_path(&path)?;
            config::initialize_config(&path)?;
            println!("Initialized rotten to {path:?}");
            Ok(())
        }
        cli::Commands::Add { source, target } => {
            let source = utils::parse_path(&source)?;
            let root =
                config::get_config_path().expect("You need to initialize rotten before adding");

            let root_path = PathBuf::from(&root);
            if !root_path.exists() {
                anyhow::bail!("Rotten root folder doens't exists, initialize");
            }
            if !root_path.join("rotten.toml").exists() {
                anyhow::bail!("rotten.toml don't exist in root")
            }

            let target = format!("{root}/{target}");
            let target = utils::parse_path(&target)?;
            utils::copy_recursive(&source, &target)?;
            Ok(())
        }
        cli::Commands::Link => {
            todo!("Link all to places");
        }
    }
}
