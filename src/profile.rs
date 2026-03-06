use anyhow::{bail, Context, Result};
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};

use crate::state;

pub fn profiles_dir() -> PathBuf {
    dirs::home_dir()
        .expect("could not find home directory")
        .join(".agents")
        .join("profiles")
}

pub fn skills_dir() -> PathBuf {
    dirs::home_dir()
        .expect("could not find home directory")
        .join(".agents")
        .join("skills")
}

pub fn list_profiles() -> Result<Vec<String>> {
    let dir = profiles_dir();
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut names: Vec<String> = fs::read_dir(&dir)
        .with_context(|| format!("failed to read profiles dir: {}", dir.display()))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().ok()?.is_dir() {
                entry.file_name().into_string().ok()
            } else {
                None
            }
        })
        .collect();
    names.sort();
    Ok(names)
}

pub fn current_profile() -> Option<String> {
    let skills = skills_dir();
    let target = fs::read_link(&skills).ok()?;
    // Resolve to absolute path so we can get the last component reliably
    let resolved = if target.is_absolute() {
        target
    } else {
        skills.parent()?.join(target)
    };
    resolved
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
}

pub fn switch_to(name: &str) -> Result<()> {
    let profiles = profiles_dir();
    let profile_path = profiles.join(name);
    if !profile_path.exists() {
        bail!("profile '{}' does not exist", name);
    }

    let skills = skills_dir();

    // Save current profile before switching
    if let Some(current) = current_profile() {
        if current != name {
            state::write_previous(&current);
        }
    }

    // Remove existing symlink or file at skills_dir
    if skills.exists() || skills.symlink_metadata().is_ok() {
        fs::remove_file(&skills)
            .with_context(|| format!("failed to remove existing skills link: {}", skills.display()))?;
    }

    unix_fs::symlink(&profile_path, &skills)
        .with_context(|| format!("failed to create symlink {} -> {}", skills.display(), profile_path.display()))?;

    Ok(())
}

pub fn create_from_path(name: &str, source: Option<&Path>) -> Result<()> {
    let profiles = profiles_dir();
    let dest = profiles.join(name);
    if dest.exists() {
        bail!("profile '{}' already exists", name);
    }

    let resolved_source: PathBuf = match source {
        Some(p) => p.canonicalize().with_context(|| format!("cannot resolve path: {}", p.display()))?,
        None => {
            let skills = skills_dir();
            skills.canonicalize().with_context(|| {
                format!(
                    "cannot resolve skills dir '{}' — is a profile active?",
                    skills.display()
                )
            })?
        }
    };

    fs::create_dir_all(&dest)
        .with_context(|| format!("failed to create profile dir: {}", dest.display()))?;

    // Copy immediate subdirectories (skill dirs)
    let mut copied = 0;
    for entry in fs::read_dir(&resolved_source)
        .with_context(|| format!("failed to read source dir: {}", resolved_source.display()))?
    {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let skill_name = entry.file_name();
            copy_dir_recursive(&entry.path(), &dest.join(&skill_name))?;
            copied += 1;
        }
    }

    println!("Created profile '{}' with {} skill(s) from '{}'", name, copied, resolved_source.display());
    println!("Run `rt {}` to switch to it.", name);
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &dst_path)?;
        } else {
            fs::copy(&entry.path(), &dst_path)?;
        }
    }
    Ok(())
}

pub fn delete_profile(name: &str) -> Result<()> {
    let profile_path = profiles_dir().join(name);
    if !profile_path.exists() {
        bail!("profile '{}' does not exist", name);
    }
    // Refuse to delete the currently active profile
    if current_profile().as_deref() == Some(name) {
        bail!("cannot delete the currently active profile '{}' — switch away first", name);
    }
    fs::remove_dir_all(&profile_path)
        .with_context(|| format!("failed to delete profile '{}'", name))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiles_dir_path() {
        let path = profiles_dir();
        assert!(path.ends_with(".agents/profiles"));
    }

    #[test]
    fn test_skills_dir_path() {
        let path = skills_dir();
        assert!(path.ends_with(".agents/skills"));
    }
}
