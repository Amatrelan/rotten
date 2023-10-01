use std::{collections::HashMap, io::prelude::*, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default)]
pub struct ConfigManager {
    state_file: PathBuf,
    config_file: PathBuf,
}

impl ConfigManager {
    #[tracing::instrument]
    pub fn try_new(new_path: &PathBuf) -> anyhow::Result<Self> {
        let state = get_state_path()?;
        tracing::info!("State file: {:?}", state);

        std::fs::create_dir_all(&new_path)?;
        let config_path = new_path.join("rotten.toml");

        // This here just we create config file and that's it.
        {
            tracing::info!("Creating empty config");
            let mut f = std::fs::File::create(&config_path).expect("Failed to create config file");
            let c = Config::default();
            let c = toml_edit::ser::to_string(&c).expect("Failed to serialize default config");
            f.write_all(c.as_bytes())?;
        }

        let mut f = std::fs::File::create(&state).expect("Failed to create state file");
        f.write_all(&config_path.to_str().unwrap().as_bytes())?;

        Ok(Self {
            state_file: state,
            config_file: config_path,
        })
    }

    #[tracing::instrument]
    pub fn try_load() -> anyhow::Result<Self> {
        let state = get_state_path()?;

        tracing::info!("State file: {:?}", state);

        let config_file = Self::get_config_path(&state)?;

        Ok(Self {
            state_file: state,
            config_file,
        })
    }

    #[tracing::instrument]
    pub fn get_config(&self) -> anyhow::Result<Config> {
        let f =
            std::fs::read_to_string(&self.config_file).expect("Failed to read config file content");

        let a: Config = toml_edit::de::from_str(&f).expect("Failed to read config file");

        return Ok(a);
    }

    pub fn add_link(&mut self, name: String, link: Symlink) -> anyhow::Result<()> {
        let mut config_data = {
            let data = std::fs::read_to_string(&self.config_file)?;
            let toml: toml_edit::Document = data.parse()?;
            toml
        };

        let links = config_data["links"]
            .as_table_mut()
            .expect("Links wasn't table");
        let target = link.target.to_str().unwrap();
        let source = link.source.to_str().unwrap();
        links[&name] = toml_edit::table();
        links[&name]["source"] = toml_edit::value(target);
        links[&name]["target"] = toml_edit::value(source);

        self.write_config(config_data);

        Ok(())
    }

    pub fn write_config(&mut self, config: toml_edit::Document) {
        let mut f = std::fs::File::create(&self.config_file).expect("Failed to create config file");

        let config = config.to_string();
        f.write_all(config.as_bytes())
            .expect("Failed to write config file");
    }

    #[tracing::instrument]
    pub fn set_config_path(&self, p: &str) {
        let mut f =
            std::fs::File::create(&self.state_file).expect("Failed to open rotten state file");
        f.write_all(p.as_bytes())
            .expect("Failed to write to state file");
    }

    fn get_config_path(state: &PathBuf) -> anyhow::Result<PathBuf> {
        let state_data = std::fs::read_to_string(&state);

        match state_data {
            Err(e) => {
                return anyhow::bail!("Failed to read state file: {e}");
            }
            Ok(val) => match val.lines().next() {
                None => anyhow::bail!("State file don't have config path"),
                Some(line) => {
                    let file_path = std::path::PathBuf::from(line);
                    match file_path.exists() {
                        true => Ok(file_path),
                        false => anyhow::bail!("Config file doesn't exist what is in state file"),
                    }
                }
            },
        }
    }
}
fn get_state_path() -> anyhow::Result<PathBuf> {
    let config_path = if let Ok(xsd_state_home) = std::env::var("XSD_STATE_HOME") {
        xsd_state_home
    } else {
        let home = std::env::var("HOME")?;
        format!("{home}/.local/state")
    };

    Ok(PathBuf::from(config_path).join("rotten"))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub terminal: Option<String>,
    pub links: HashMap<String, LinkConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkConfig {
    #[serde(flatten)]
    pub symlink: Symlink,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Symlink {
    pub source: PathBuf,
    pub target: PathBuf,
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
source = "nvim" # path in rotten folder what is linked
target = "$XDG_CONFIG_HOME/nvim" # where source is linked

[links.emacs]
source = "emacs/emacs"
target = "$XDG_CONFIG_HOME/emacs"

[links.doom]
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
