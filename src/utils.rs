pub fn parse_path(path: &str) -> anyhow::Result<std::path::PathBuf> {
    let path = path.trim();
    let split_path: Vec<&str> = path.split('/').collect();

    let mut new_path: Vec<String> = vec![];
    for each in &split_path {
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
    let paths = std::fs::read_dir(&from)?;
    std::fs::create_dir_all(&to)?;

    for path in paths {
        let path = path?;
        if path.metadata()?.is_dir() {
            let from_new_path = from.join(path.path().file_name().unwrap());
            let to_new_path = to.join(path.path().file_name().unwrap());
            copy_recursive(&from_new_path, &to_new_path)?;
            continue;
        }

        if path.metadata()?.is_file() {
            let from_path = from.join(&path.path().file_name().unwrap());
            let to_path = to.join(&path.path().file_name().unwrap());
            std::fs::copy(&from_path, &to_path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_path() {
        //
    }
}
