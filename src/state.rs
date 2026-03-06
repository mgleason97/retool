use std::fs;
use std::path::PathBuf;

pub fn state_file() -> PathBuf {
    dirs::home_dir()
        .expect("could not find home directory")
        .join(".agents")
        .join(".retool_previous")
}

pub fn read_previous() -> Option<String> {
    fs::read_to_string(state_file())
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn write_previous(name: &str) {
    let path = state_file();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, name);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_file_path() {
        let path = state_file();
        assert!(path.ends_with(".agents/.retool_previous"));
    }
}
