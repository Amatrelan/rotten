use color_eyre::{eyre::eyre, Result};

pub fn parse_path(path: &std::path::Path) -> Result<std::path::PathBuf> {
    log::trace!("Finding correct path");
    let mut new_path: Vec<String> = vec![];
    for each in path.components() {
        let each = each
            .as_os_str()
            .to_str()
            .ok_or(eyre!("Failed to convert {each:?} to str"))?;
        if each == "~" {
            let val = std::env::var("HOME")?;
            new_path.push(val);
            continue;
        }
        if let Some(stripped) = each.strip_prefix('$') {
            let val = std::env::var(stripped)?;
            new_path.push(val);
            continue;
        }

        new_path.push(each.to_string());
    }

    let mut path = new_path.join("/");
    if path.starts_with("//") {
        path = path[1..].to_string();
    }
    let path = std::path::PathBuf::from(path);
    log::trace!("New path is {path:?}");
    Ok(path)
}

pub fn copy_recursive(from: &std::path::PathBuf, to: &std::path::PathBuf) -> Result<()> {
    log::trace!("Copying file recursive");
    if from.is_dir() {
        std::fs::create_dir_all(to)?;
    }
    if from.is_dir() {
        let paths = std::fs::read_dir(from)?;
        for path in paths {
            let path = path?;
            let file_name = path.path();
            let file_name = file_name
                .file_name()
                .ok_or(eyre!("Failed to get file name from {path:?}"))?;

            if path.metadata()?.is_dir() {
                let from_new_path = from.join(&file_name);
                let to_new_path = to.join(&file_name);
                copy_recursive(&from_new_path, &to_new_path)?;
                continue;
            }

            if path.metadata()?.is_file() {
                let from_path = from.join(&file_name);
                let to_path = to.join(&file_name);
                if to_path.exists() {
                    return Err(eyre!("File {to_path:?} already exists"));
                }
                log::info!("Copying {from_path:?} to {to_path:?}");
                std::fs::copy(&from_path, &to_path)?;
            }
        }
    } else {
        if to.exists() {
            return Err(eyre!("File {to:?} already exists"));
        }
        std::fs::copy(from, to)?;
    }

    Ok(())
}
