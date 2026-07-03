use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

// auth.json 的变更指纹（mtime + 大小），用于感知账号切换工具改写凭据文件。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct AuthFileState {
    modified_at: Option<SystemTime>,
    size: Option<u64>,
}

pub(super) fn read_auth_file_state() -> AuthFileState {
    codex_auth_file_path()
        .map(|path| auth_file_state_at(&path))
        .unwrap_or_default()
}

fn auth_file_state_at(path: &Path) -> AuthFileState {
    match fs::metadata(path) {
        Ok(metadata) => AuthFileState {
            modified_at: metadata.modified().ok(),
            size: Some(metadata.len()),
        },
        Err(_) => AuthFileState::default(),
    }
}

fn codex_auth_file_path() -> Option<PathBuf> {
    codex_home_dir_from(env::var_os("CODEX_HOME"), home_dir_var()).map(|dir| dir.join("auth.json"))
}

fn home_dir_var() -> Option<OsString> {
    #[cfg(windows)]
    {
        env::var_os("USERPROFILE")
    }
    #[cfg(not(windows))]
    {
        env::var_os("HOME")
    }
}

fn codex_home_dir_from(codex_home: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    if let Some(value) = codex_home {
        if !value.is_empty() {
            return Some(PathBuf::from(value));
        }
    }
    home.map(|value| PathBuf::from(value).join(".codex"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 优先使用_codex_home_环境变量() {
        let dir = codex_home_dir_from(
            Some(OsString::from("/custom/codex-home")),
            Some(OsString::from("/home/test")),
        );

        assert_eq!(dir, Some(PathBuf::from("/custom/codex-home")));
    }

    #[test]
    fn codex_home_为空时回退到用户目录() {
        let dir = codex_home_dir_from(Some(OsString::new()), Some(OsString::from("/home/test")));

        assert_eq!(dir, Some(PathBuf::from("/home/test").join(".codex")));
    }

    #[test]
    fn 缺少用户目录时返回_none() {
        assert_eq!(codex_home_dir_from(None, None), None);
    }

    #[test]
    fn 文件不存在时返回默认指纹() {
        let path = env::temp_dir().join("codex-widget-auth-missing-test.json");

        assert_eq!(auth_file_state_at(&path), AuthFileState::default());
    }

    #[test]
    fn 文件内容变化后指纹不同() {
        let path = env::temp_dir().join(format!(
            "codex-widget-auth-fingerprint-{}.json",
            std::process::id()
        ));
        fs::write(&path, b"account-a").unwrap();
        let first = auth_file_state_at(&path);
        fs::write(&path, b"account-b-longer").unwrap();
        let second = auth_file_state_at(&path);
        let _ = fs::remove_file(&path);

        assert_ne!(first, AuthFileState::default());
        assert_ne!(first, second);
    }
}
