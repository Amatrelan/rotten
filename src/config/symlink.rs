use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Symlink {
    pub from: PathBuf,
    pub to: PathBuf,
}

impl std::fmt::Display for Symlink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let from_str = self.from.to_str().unwrap();
        let to_str = self.to.to_str().unwrap();
        write!(f, "Symlink: {from_str} => {to_str}")
    }
}

impl Symlink {
    pub fn new(from: PathBuf, to: PathBuf) -> Self {
        Self { from, to }
    }

    pub fn link(&self, overwrite: bool, config_root: &Path) -> anyhow::Result<()> {
        let to_pathbuf = self.get_to();
        let to_path = to_pathbuf.to_str().unwrap();

        let from_pathbuf = self.get_from(&config_root);
        if !from_pathbuf.exists() {
            let from_path = from_pathbuf.to_str().unwrap();
            anyhow::bail!("\"{from_path} doesn't exist\"");
        }

        if let Ok(a) = std::fs::read_link(&to_pathbuf) {
            if a == from_pathbuf {
                log::info!("{from_pathbuf:?} already points to correct place, skipping");
                return Ok(());
            }
        }

        if to_pathbuf.exists() {
            if !overwrite {
                anyhow::bail!("\"{to_path}\" exists");
            }

            if self.is_dir(&config_root) {
                if let Err(e) = std::fs::remove_dir_all(&to_pathbuf) {
                    anyhow::bail!("\"{to_path}\" removal failed: {e}")
                }
            } else {
                if let Err(e) = std::fs::remove_file(&to_pathbuf) {
                    anyhow::bail!("\"{to_path}\" removal failed: {e}")
                }
            }
        }

        if let Err(e) = std::os::unix::fs::symlink(&from_pathbuf, &to_pathbuf) {
            anyhow::bail!("Failed: {self} :: {e}");
        }

        Ok(())
    }

    fn get_to(&self) -> PathBuf {
        crate::utils::parse_path(&self.to).unwrap()
    }

    fn get_from(&self, config_root: &Path) -> PathBuf {
        config_root.join(&self.from)
    }

    fn is_dir(&self, config_root: &Path) -> bool {
        self.get_from(config_root).is_dir()
    }
}
