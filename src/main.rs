/// Simple tool to symlink dotfiles
///
/// # TODO list
/// - add md5 checksumming and validate files after.
use clap::Parser;
mod cli;
mod config;
mod logging;
mod utils;

use color_eyre::eyre::Result;

#[cfg(unix)]
fn main() -> Result<()> {
    let _ = color_eyre::install();
    let matches = cli::Cli::parse();
    let log_level: log::LevelFilter = match matches.verbosity {
        1 => log::LevelFilter::Error,
        2 => log::LevelFilter::Warn,
        3 => log::LevelFilter::Info,
        4 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    logging::set_logger(log_level);

    match matches.command {
        cli::Commands::Init { overwrite } => {
            let path = std::env::current_dir()?;
            let mut cm = config::ConfigManager::try_new(&path, overwrite)
                .expect("Failed to setup state file");
            cm.setup_config().expect("Failed to setup config file");
            log::info!("Initialized rotten to {path:?}");
            Ok(())
        }
        cli::Commands::Setup => {
            let path = std::env::current_dir()?;
            let current = std::env::current_dir()?;
            if !current.join("rotten.toml").exists() {
                log::error!("Current folder ({path:?}) isn't rotten folder");
                std::process::exit(1);
            }

            config::ConfigManager::try_new(&path, false).expect("Failed to setup state file");

            Ok(())
        }
        cli::Commands::Link {
            overwrite,
            profiles,
        } => {
            let cm = config::ConfigManager::try_load().expect("Failed to read config");

            for each in profiles {
                log::info!("Symlinking profile: {each}");
                cm.symlink_profile(overwrite, each)?;
            }

            Ok(())
        }
        cli::Commands::Unlink => {
            todo!("Create unlinker")
        }
    }
}

#[cfg(not(unix))]
fn main() {
    eprintln!("Sorry not working on non unix right now");
}
