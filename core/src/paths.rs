use std::env;
use std::path::{Path, PathBuf};

pub fn data_dir() -> PathBuf {
    if let Some(value) = env::var_os("DUBBIT_HOME") {
        return PathBuf::from(value);
    }
    if cfg!(windows) {
        let base = env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| home().join("AppData").join("Local"));
        return base.join("Dubbit");
    }
    if cfg!(target_os = "macos") {
        return home()
            .join("Library")
            .join("Application Support")
            .join("Dubbit");
    }
    let base = env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".local").join("share"));
    base.join("dubbit")
}

fn home() -> PathBuf {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn config_file() -> PathBuf {
    data_dir().join("config.json")
}

pub fn tools_dir() -> PathBuf {
    data_dir().join("bin")
}

pub fn temp_dir() -> PathBuf {
    env::temp_dir().join("dubbit")
}

pub fn models_dir() -> PathBuf {
    data_dir().join("models")
}

pub fn state_file() -> PathBuf {
    data_dir().join("state.json")
}

pub fn reports_dir() -> PathBuf {
    data_dir().join("reports")
}

pub fn tool_dirs() -> Vec<PathBuf> {
    let mut directories = Vec::new();
    if let Some(value) = env::var_os("DUBBIT_TOOLS_DIR") {
        directories.push(PathBuf::from(value));
    }
    directories.push(tools_dir());
    if let Ok(executable) = env::current_exe() {
        if let Some(parent) = executable.parent() {
            directories.push(parent.to_path_buf());
        }
    }
    if let Ok(current) = env::current_dir() {
        directories.push(current.join(".tools").join("bin"));
    }
    directories
}

pub fn find_binary(name: &str, env_var: &str) -> Option<PathBuf> {
    if let Some(value) = env::var_os(env_var) {
        let candidate = PathBuf::from(value);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    let mut names = vec![name.to_string()];
    if cfg!(windows) && !name.ends_with(".exe") {
        names.push(format!("{name}.exe"));
    }
    for directory in tool_dirs() {
        for candidate in &names {
            let path = directory.join(candidate);
            if path.is_file() {
                return Some(path);
            }
        }
    }
    let search = env::var_os("PATH")?;
    for directory in env::split_paths(&search) {
        for candidate in &names {
            let path = directory.join(candidate);
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

pub fn relative_to(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
