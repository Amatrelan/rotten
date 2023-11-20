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
