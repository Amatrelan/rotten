/// Simple tool to symlink dotfiles
///
/// # TODO list
/// - add md5 checksumming and validate files after.
use clap::Parser;

mod cli;
mod config;
mod log;
mod utils;

#[cfg(unix)]
fn main() -> anyhow::Result<()> {
    let matches = cli::Cli::parse();

    log::set_logger();

    match matches.command {
        cli::Commands::Init => {
            let path = std::env::current_dir()?;
            let cm = config::ConfigManager::try_new(&path).expect("Failed to setup state file");
            cm.setup_config().expect("Failed to setup config file");
            println!("Initialized rotten to {path:?}");
            Ok(())
        }
        cli::Commands::Setup => {
            let path = std::env::current_dir()?;
            config::ConfigManager::try_new(&path).expect("Failed to setup state file");
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

            if !source.exists() {
                anyhow::bail!("Source {source:?} doesn't exist");
            }

            println!("Creating link `{:?}` => `{:?}`", &source_full, &target);
            let sym = config::Symlink {
                source,
                target: std::path::PathBuf::from(&target),
            };
            cm.add_link(name, sym)?;

            let config_dir = cm.config_path;
            let target = config_dir.join(target);
            utils::copy_recursive(&source_full, &target)?;

            Ok(())
        }
        cli::Commands::Link { overwrite } => {
            let cm = config::ConfigManager::try_load().expect("Failed to read config");
            let config = cm.get_config()?;

            for (key, value) in config.links {
                if let Some(disabled) = value.disabled {
                    if disabled {
                        println!("\"{key}\" was disabled, skipping");
                        continue;
                    }
                }

                let source = value.symlink.source;
                let source = cm.config_path.join(source);

                let target = value.symlink.target;
                let target = utils::parse_path(&target)?;

                let is_symlink = std::fs::symlink_metadata(&target)?.is_symlink();
                if target.exists() && !is_symlink {
                    if !overwrite {
                        eprintln!("{target:?} already exists and isn't symlink, move it");
                        std::process::exit(1);
                    }

                    println!("Removing old link {target:?}");
                    if target.metadata().unwrap().is_dir() {
                        std::fs::remove_dir_all(&target).expect("Failed to remove {target}");
                    } else {
                        std::fs::remove_file(&target).expect("Failed to remove {target}");
                    }
                }

                println!("Linking {key}: {source:?} => {target:?}");
                std::os::unix::fs::symlink(source, target)
                    .expect("Failed to symlink {source} to {target}");
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
