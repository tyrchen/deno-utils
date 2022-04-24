// Copyright 2018-2022 the Deno authors. All rights reserved. MIT license.

use deno_core::anyhow::Context;
use deno_core::error::AnyError;
use std::env::current_dir;
use std::hash::{Hash, Hasher};
use std::io::Error;
use std::path::{Component, Path, PathBuf};

/// Similar to `std::fs::canonicalize()` but strips UNC prefixes on Windows.
pub fn canonicalize_path(path: &Path) -> Result<PathBuf, Error> {
    let mut canonicalized_path = path.canonicalize()?;
    if cfg!(windows) {
        canonicalized_path = PathBuf::from(
            canonicalized_path
                .display()
                .to_string()
                .trim_start_matches("\\\\?\\"),
        );
    }
    Ok(canonicalized_path)
}

pub fn resolve_from_cwd(path: &Path) -> Result<PathBuf, AnyError> {
    let resolved_path = if path.is_absolute() {
        path.to_owned()
    } else {
        let cwd = current_dir().context("Failed to get current working directory")?;
        cwd.join(path)
    };

    Ok(normalize_path(&resolved_path))
}

/// Normalize all intermediate components of the path (ie. remove "./" and "../" components).
/// Similar to `fs::canonicalize()` but doesn't resolve symlinks.
///
/// Taken from Cargo
/// <https://github.com/rust-lang/cargo/blob/af307a38c20a753ec60f0ad18be5abed3db3c9ac/src/cargo/util/paths.rs#L60-L85>
pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut components = path.as_ref().components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}

pub fn to_hash_path(base: &Path, key: &str) -> PathBuf {
    let hash = get_hash_from_key(key);
    base.join(format!("{}/{}/{}", &hash[..2], &hash[2..4], &hash[4..]))
}

fn get_hash_from_key(key: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_from_cwd_child() {
        let cwd = current_dir().unwrap();
        assert_eq!(resolve_from_cwd(Path::new("a")).unwrap(), cwd.join("a"));
    }

    #[test]
    fn resolve_from_cwd_dot() {
        let cwd = current_dir().unwrap();
        assert_eq!(resolve_from_cwd(Path::new(".")).unwrap(), cwd);
    }

    #[test]
    fn resolve_from_cwd_parent() {
        let cwd = current_dir().unwrap();
        assert_eq!(resolve_from_cwd(Path::new("a/..")).unwrap(), cwd);
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path(Path::new("a/../b")), PathBuf::from("b"));
        assert_eq!(normalize_path(Path::new("a/./b/")), PathBuf::from("a/b/"));
        assert_eq!(
            normalize_path(Path::new("a/./b/../c")),
            PathBuf::from("a/c")
        );

        if cfg!(windows) {
            assert_eq!(
                normalize_path(Path::new("C:\\a\\.\\b\\..\\c")),
                PathBuf::from("C:\\a\\c")
            );
        }
    }

    // TODO: Get a good expected value here for Windows.
    #[cfg(not(windows))]
    #[test]
    fn resolve_from_cwd_absolute() {
        let expected = Path::new("/a");
        assert_eq!(resolve_from_cwd(expected).unwrap(), expected);
    }
}
