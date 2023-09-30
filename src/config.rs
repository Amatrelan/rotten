use std::{collections::HashMap, io::prelude::*, path::PathBuf};

use serde::{Deserialize, Serialize};

/// Use this when just reading config
pub fn get_config(s: &str) -> anyhow::Result<Config> {
    let a = toml_edit::de::from_str::<Config>(s).unwrap();
    Ok(a)
}

/// Use this when editing config, so comments are kept in place and not removed
pub fn get_value_mut(s: &str) -> anyhow::Result<toml_edit::Value> {
    let a: toml_edit::Value = s.parse::<toml_edit::Value>().unwrap();
    Ok(a)
}

pub fn write_config(s: toml_edit::Value) -> anyhow::Result<()> {
    panic!()
}

pub fn initialize_config(path: &PathBuf) -> anyhow::Result<()> {
    let path = PathBuf::from(path);
    let rotten_path = path.join("rotten.toml");

    if rotten_path.exists() {
        anyhow::bail!("{path:?} exists, move rotten.toml to regenerate");
    }
    setup_state(&path)?;
    std::fs::create_dir_all(&path)?;

    let mut file = std::fs::File::create(rotten_path)?;
    let config = generate_commented_empty();
    file.write_all(config.as_bytes())?;

    Ok(())
}

fn setup_state(path: &PathBuf) -> anyhow::Result<()> {
    let state_file = get_state()?;
    let path = path.to_str().unwrap();
    let mut file = std::fs::File::create(state_file)?;
    // This is not safe in Windows, so have fun
    file.write_all(path.as_bytes())?;
    Ok(())
}

pub fn get_config_path() -> anyhow::Result<String> {
    let state = get_state()?;
    Ok(std::fs::read_to_string(state)?.trim().to_string())
}

fn get_state() -> anyhow::Result<PathBuf> {
    let config_path = if let Ok(xsd_state_home) = std::env::var("XSD_STATE_HOME") {
        xsd_state_home
    } else {
        // If HOME don't exist, panic, I don't care that much about windows at this stage
        // and in linux it 99% time is set, and for me this works.
        // TODO: Fix this some day, perhaps if some people ask for this
        let home = std::env::var("HOME")?;
        format!("{home}/.local/state")
    };

    Ok(PathBuf::from(config_path).join("rotten_state"))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub terminal: Option<String>,
    pub links: HashMap<String, LinkConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkConfig {
    pub name: Option<String>,
    #[serde(flatten)]
    pub symlink: Symlink,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Symlink {
    pub source: String,
    pub target: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            terminal: None,
            links: HashMap::new(),
        }
    }
}

fn generate_empty() -> &'static str {
    return r#"terminal = "bash" # Use this shell, defaults to `$SHELL` if not set

[links.nvim]
name = "nvim" # Optional, this is used only when logging
source = "nvim" # path in rotten folder what is linked
target = "$XDG_CONFIG_HOME/nvim" # where source is linked

[links.emacs]
name = "emacs"
source = "emacs/emacs"
target = "$XDG_CONFIG_HOME/emacs"

[links.doom]
name = "doom-emacs"
source = "emacs/doom"
target = "$XDG_CONFIG_HOME/doom""#;
}

fn generate_commented_empty() -> String {
    generate_empty()
        .split('\n')
        .map(|a| {
            return format!("# {a}\n");
        })
        .collect()
}

#[cfg(test)]
mod config {
    use super::*;

    #[test]
    fn validate_example() {
        let template = generate_empty();
        toml_edit::de::from_str::<Config>(&template).unwrap();
    }
}
