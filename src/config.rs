use color_eyre::eyre::{eyre, Result};
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
    pub fn try_new(config_path: &PathBuf, overwrite: bool) -> Result<Self> {
        let state_path = get_state_path()?;

        log::info!("State file: {:?}", state_path);

        let mut f = std::fs::File::create(&state_path)?;

        f.write_all(
            config_path
                .to_str()
                .ok_or(eyre!("Failed to write to {config_path:?}"))?
                .as_bytes(),
        )?;

        let mut new = Self {
            state_path,
            config_root: config_path.clone(),
            config: None,
        };

        if overwrite {
            new.setup_config()?;
        }

        Ok(new)
    }

    pub fn try_load() -> Result<Self> {
        let state = get_state_path()?;
        log::info!("State file: {state:?}");
        let config_root = Self::get_config_root(&state)?;
        log::info!("Config root: {config_root:?}");

        let mut a = Self {
            state_path: state,
            config_root,
            config: None,
        };

        a.get_config()?;

        Ok(a)
    }

    pub fn setup_config(&mut self) -> Result<()> {
        let config_path = self.config_root.join("rotten.toml");
        let mut f = std::fs::File::create(config_path)?;
        let c = generate_empty();
        f.write_all(c.as_bytes())?;

        Ok(())
    }

    fn get_config(&mut self) -> Result<()> {
        let f = std::fs::read_to_string(self.config_root.join("rotten.toml"))
            .expect("Failed to read config file content");

        let a: Config = toml_edit::de::from_str(&f).expect("Failed to read config file");
        self.config = Some(a);
        Ok(())
    }

    pub fn add_link(&mut self, name: String, link: Symlink) -> Result<()> {
        let mut config_data = {
            let data = std::fs::read_to_string(self.config_root.join("rotten.toml"))?;
            let toml: toml_edit::Document = data.parse()?;
            toml
        };

        let links = config_data["links"]
            .as_table_mut()
            .expect("Links wasn't table");
        let target = link
            .to
            .to_str()
            .ok_or(eyre!("Failed to convert {link:?}.to to str"))?;
        let source = link
            .from
            .to_str()
            .ok_or(eyre!("Failed to convert {link:?}.from to str"))?;
        links[&name] = toml_edit::table();
        links[&name]["source"] = toml_edit::value(target);
        links[&name]["target"] = toml_edit::value(source);

        self.write_config(config_data);

        Ok(())
    }

    pub fn symlink_profile(&self, overwrite: bool, profile: String) -> Result<()> {
        if let Some(config) = &self.config {
            if let Some(profiles) = &config.profiles {
                let profile_tools = profiles
                    .get(&profile)
                    .ok_or(eyre!("Failed to get profile: {profile}"))?;

                for active in profile_tools {
                    config
                        .links
                        .get(active)
                        .ok_or(eyre!("No symlink {active}"))?
                        .symlink
                        .link(overwrite, &self.config_root)?;
                }
            }
        }

        Ok(())
    }

    pub fn write_config(&mut self, config: toml_edit::Document) {
        let mut f = std::fs::File::create(self.config_root.join("rotten.toml"))
            .expect("Failed to create config file");

        let config = config.to_string();
        f.write_all(config.as_bytes())
            .expect("Failed to write config file");
    }

    pub fn get_config_root(state: &PathBuf) -> Result<PathBuf> {
        let state_data = std::fs::read_to_string(state)?;

        if let Some(line) = state_data.lines().next() {
            let file_path = std::path::PathBuf::from(line);
            match file_path.exists() {
                true => return Ok(file_path),
                false => return Err(eyre!("Config file doesn't exist what is in state file")),
            }
        }

        Err(eyre!("No state file found"))
    }
}
fn get_state_path() -> Result<PathBuf> {
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
    pub profiles: Option<HashMap<String, Vec<String>>>,
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
[profiles]
example = ['nvim']

[links.nvim]
disabled = true # You can disable also some links
from = "nvim" # path in rotten folder what is linked
to = "$HOME/.config/nvim" # where source is linked
"#
}

#[cfg(test)]
mod config {
    use super::*;

    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn validate_example() {
        let template = generate_empty();
        let a = toml_edit::de::from_str::<Config>(template).unwrap();
        let mut expected = HashMap::new();
        expected.insert("example".to_string(), vec!["nvima".to_string()]);
        assert_eq!(a.profiles.unwrap(), expected)
    }
}
