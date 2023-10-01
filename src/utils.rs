pub fn parse_path(path: &std::path::PathBuf) -> anyhow::Result<std::path::PathBuf> {
    let mut new_path: Vec<String> = vec![];
    for each in path.components() {
        let each = each.as_os_str().to_str().unwrap();
        if each.starts_with('$') {
            let val = &each[1..];
            let val = std::env::var(val)?;
            new_path.push(val);
            continue;
        }

        new_path.push(each.to_string());
    }

    let path = new_path.join("/");
    let path = std::env::current_dir()?.join(path);
    Ok(path)
}

pub fn copy_recursive(from: &std::path::PathBuf, to: &std::path::PathBuf) -> anyhow::Result<()> {
    if from.is_dir() {
        let paths = std::fs::read_dir(&from)?;
        for path in paths {
            let path = path?;
            if path.metadata()?.is_dir() {
                std::fs::create_dir_all(&path.path())?;
                let from_new_path = from.join(path.path().file_name().unwrap());
                let to_new_path = to.join(path.path().file_name().unwrap());
                copy_recursive(&from_new_path, &to_new_path)?;
                continue;
            }

            if path.metadata()?.is_file() {
                let from_path = from.join(&path.path().file_name().unwrap());
                let to_path = to.join(&path.path().file_name().unwrap());
                if to_path.exists() {
                    anyhow::bail!("File {to_path:?} already exists");
                }
                println!("Copying {from_path:?} to {to_path:?}");
                std::fs::copy(&from_path, &to_path)?;
            }
        }
    } else {
        if to.exists() {
            anyhow::bail!("File {to:?} already exists");
        }
        std::fs::copy(&from, &to)?;
    }

    Ok(())
}
