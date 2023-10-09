/// Simple tool to symlink dotfiles
///
/// # TODO list
/// - add md5 checksumming and validate files after.
use clap::Parser;

mod cli;
mod config;
mod logging;
mod utils;

#[cfg(unix)]
fn main() -> anyhow::Result<()> {
    let matches = cli::Cli::parse();

    let log_level: log::LevelFilter = match matches.verbosity {
        1 => log::LevelFilter::Error,
        2 => log::LevelFilter::Warn,
        3 => log::LevelFilter::Info,
        4 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    logging::set_logger(log_level);

    log::trace!("Starting {:?}", matches);
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
        cli::Commands::Add {
            source,
            target,
            name,
        } => {
            let source = std::path::PathBuf::from(&source);
            let Ok(source_full) = utils::parse_path(&source) else {
                panic!("Failed to get full source path");
            };
            let Ok(mut cm) = config::ConfigManager::try_load() else {
                panic!("Failed to load config manager");
            };

            let name = if let Some(name) = name {
                name
            } else {
                let source = source.to_str().unwrap();
                source.split('/').last().unwrap().to_string()
            };

            let symlink = config::Symlink::new(source, std::path::PathBuf::from(&target));
            log::info!("Creating {symlink}");
            cm.add_link(name, symlink)?;

            let config_dir = cm.config_root;
            let target = config_dir.join(target);
            utils::copy_recursive(&source_full, &target).expect("Failed to copy recursive");

            Ok(())
        }
        cli::Commands::Link { overwrite } => {
            let cm = config::ConfigManager::try_load().expect("Failed to read config");
            let config = cm.get_config()?;

            for (key, value) in config.links {
                if let Some(disabled) = value.disabled {
                    if disabled {
                        log::info!("\"{key}\" was disabled, skipping");
                        continue;
                    }
                }

                if let Err(e) = value.symlink.link(overwrite, &cm.config_root) {
                    log::error!("{e}");
                }
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
