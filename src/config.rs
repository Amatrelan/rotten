use serde::{Deserialize, Serialize};
use std::io::Write;
use std::{collections::HashMap, path::PathBuf};

mod symlink;

pub use symlink::Symlink;

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct ConfigManager {
    pub state_path: PathBuf,
    pub config_root: PathBuf,
    config: Option<Config>,
}

impl ConfigManager {
    pub fn try_new(config_path: &PathBuf, overwrite: bool) -> anyhow::Result<Self> {
        let state_path = match get_state_path() {
            Ok(val) => val,
            Err(e) => anyhow::bail!("Failed to get state path: {e}"),
        };

        log::info!("State file: {:?}", state_path);

        if state_path.exists() && !overwrite {
            anyhow::bail!("State path exists, and overwrite is not enabled");
        }

        let Ok(mut f) = std::fs::File::create(&state_path) else {
            anyhow::bail!("Failed to create state file to {state_path:?}");
        };

        if let Err(e) = f.write_all(config_path.to_str().unwrap().as_bytes()) {
            anyhow::bail!("Failed to write config path to state file: {e}");
        }

        let mut new = Self {
            state_path,
            config_root: config_path.clone(),
            config: None,
        };

        new.setup_config()?;

        Ok(new)
    }

    pub fn try_load() -> anyhow::Result<Self> {
        let state = get_state_path()?;
        log::info!("State file: {state:?}");
        let config_root = Self::get_config_root(&state)?;
        log::info!("Config root: {config_root:?}");

        Ok(Self {
            state_path: state,
            config_root,
            config: None,
        })
    }

    pub fn setup_config(&mut self) -> anyhow::Result<()> {
        let config_path = self.config_root.join("rotten.toml");

        let Ok(mut f) = std::fs::File::create(config_path) else {
            anyhow::bail!("Failed to create config file");
        };

        let c = generate_empty();

        f.write_all(c.as_bytes())?;
        Ok(())
    }

    pub fn get_config(&self) -> anyhow::Result<Config> {
        let f = std::fs::read_to_string(self.config_root.join("rotten.toml"))
            .expect("Failed to read config file content");

        let a: Config = toml_edit::de::from_str(&f).expect("Failed to read config file");

        return Ok(a);
    }

    pub fn add_link(&mut self, name: String, link: Symlink) -> anyhow::Result<()> {
        let mut config_data = {
            let data = std::fs::read_to_string(self.config_root.join("rotten.toml"))?;
            let toml: toml_edit::Document = data.parse()?;
            toml
        };

        let links = config_data["links"]
            .as_table_mut()
            .expect("Links wasn't table");
        let target = link.to.to_str().unwrap();
        let source = link.from.to_str().unwrap();
        links[&name] = toml_edit::table();
        links[&name]["source"] = toml_edit::value(target);
        links[&name]["target"] = toml_edit::value(source);

        self.write_config(config_data);

        Ok(())
    }

    pub fn write_config(&mut self, config: toml_edit::Document) {
        let mut f = std::fs::File::create(self.config_root.join("rotten.toml"))
            .expect("Failed to create config file");

        let config = config.to_string();
        f.write_all(config.as_bytes())
            .expect("Failed to write config file");
    }

    pub fn set_config_path(&self, p: &str) {
        let mut f =
            std::fs::File::create(&self.state_path).expect("Failed to open rotten state file");
        f.write_all(p.as_bytes())
            .expect("Failed to write to state file");
    }

    pub fn get_config_root(state: &PathBuf) -> anyhow::Result<PathBuf> {
        let state_data = std::fs::read_to_string(state);

        match state_data {
            Err(e) => {
                anyhow::bail!("Failed to read state file: {e}")
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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    // pub terminal: Option<String>,
    pub links: HashMap<String, LinkConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkConfig {
    pub disabled: Option<bool>,
    #[serde(flatten)]
    pub symlink: Symlink,
}

#[allow(dead_code)]
fn generate_empty() -> &'static str {
    r#"
[links.nvim]
disabled = true # You can disable also some links
from = "nvim" # path in rotten folder what is linked
to = "$HOME/.config/nvim" # where source is linked
"#
}

#[cfg(test)]
mod config {
    use super::*;

    #[test]
    fn validate_example() {
        let template = generate_empty();
        toml_edit::de::from_str::<Config>(template).unwrap();
    }
}
