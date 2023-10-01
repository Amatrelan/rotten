use std::{env::current_dir};

/// Simple tool to symlink dotfiles
///
/// # TODO list
/// - add md5 checksumming and validate files after.
use clap::Parser;

mod cli;
mod config;
mod log;
mod utils;

fn main() -> anyhow::Result<()> {
    let matches = cli::Cli::parse();

    log::set_logger();

    match matches.command {
        cli::Commands::Init => {
            let path = std::env::current_dir()?;
            let _cm = config::ConfigManager::try_new(&path)?;
            println!("Initialized rotten to {path:?}");
            Ok(())
        }
        cli::Commands::New { path } => {
            let path = std::path::PathBuf::from(path);
            let path = utils::parse_path(&path)?;
            let _cm = config::ConfigManager::try_new(&path)?;
            println!("Initialized rotten to {path:?}");
            Ok(())
        }
        cli::Commands::Add {
            source,
            target,
            name,
        } => {
            let source = std::path::PathBuf::from(&source);
            let source_full = utils::parse_path(&source)?;
            let mut cm = config::ConfigManager::try_load()?;

            let name = if let Some(name) = name {
                name
            } else {
                let source = source.to_str().unwrap();
                source.split('/').last().unwrap().to_string()
            };

            let root = current_dir().expect("Failed to get current directory");
            let target = root.join(target);
            let target = utils::parse_path(&target)?;

            if !source.exists() {
                anyhow::bail!("Source {source:?} doesn't exist");
            }

            println!("Creating link from `{:?}` to `{:?}`", &source_full, &target);
            let sym = config::Symlink {
                source,
                target: target.clone(),
            };
            cm.add_link(name, sym)?;
            utils::copy_recursive(&source_full, &target)?;

            Ok(())
        }
        cli::Commands::Link => {
            let cm = config::ConfigManager::try_load().expect("Failed to read config");
            let config = cm.get_config()?;

            for (key, value) in config.links {
                let source = value.symlink.source;
                let target = value.symlink.target;
                println!("Linking {key}: {source:?} => {target:?}");
            }

            Ok(())
        }
    }
}
