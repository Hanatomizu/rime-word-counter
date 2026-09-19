//! 路径解析模块
//!
//! 统一计算缓存目录、默认日志/数据库路径，并负责按需创建父目录。
//! CLI 与 Tauri 桌面端共用这里的逻辑，避免两边路径不一致。

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// 应用在系统缓存目录下的根目录名。
const APP_DIR_NAME: &str = "rime-word-counter";

/// 默认日志文件名（Lua 脚本也写这个路径）。
const LOG_FILE_NAME: &str = "rime_word.log";

/// 默认数据库文件名。
const DB_FILE_NAME: &str = "rime_stats.db";

/// 获取默认的缓存目录路径。
///
/// Linux: `~/.cache/rime-word-counter`
/// macOS: `~/Library/Caches/rime-word-counter`
/// Windows: `%LOCALAPPDATA%\rime-word-counter`
pub fn get_cache_dir() -> PathBuf {
    let base = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join(APP_DIR_NAME)
}

/// 获取默认的日志文件路径（Lua 脚本写入的 CSV）。
pub fn default_log_path() -> String {
    get_cache_dir()
        .join(LOG_FILE_NAME)
        .to_string_lossy()
        .to_string()
}

/// 获取默认的数据库路径。
pub fn default_db_path() -> String {
    get_cache_dir()
        .join(DB_FILE_NAME)
        .to_string_lossy()
        .to_string()
}

/// 确保目录存在（递归创建）。
pub fn ensure_dir(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir).with_context(|| format!("无法创建目录: {}", dir.display()))
}

/// 确保缓存目录存在。
pub fn ensure_cache_dir() -> Result<PathBuf> {
    let dir = get_cache_dir();
    ensure_dir(&dir)?;
    Ok(dir)
}

/// 确保某个文件路径的父目录存在。
///
/// 对 `:memory:` 这类非文件路径、以及没有父目录的相对路径直接跳过。
pub fn ensure_parent_dir(file_path: &str) -> Result<()> {
    if file_path.is_empty() || file_path == ":memory:" {
        return Ok(());
    }

    match Path::new(file_path).parent() {
        Some(parent) if !parent.as_os_str().is_empty() => ensure_dir(parent),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_paths_point_into_cache_dir() {
        let cache = get_cache_dir();
        assert!(default_log_path().starts_with(&cache.to_string_lossy().to_string()));
        assert!(default_db_path().starts_with(&cache.to_string_lossy().to_string()));
        assert!(default_log_path().ends_with(LOG_FILE_NAME));
        assert!(default_db_path().ends_with(DB_FILE_NAME));
    }

    #[test]
    fn test_ensure_parent_dir_skips_memory_and_bare_names() {
        assert!(ensure_parent_dir(":memory:").is_ok());
        assert!(ensure_parent_dir("").is_ok());
        assert!(ensure_parent_dir("stats.db").is_ok());
    }

    #[test]
    fn test_ensure_parent_dir_creates_nested_dirs() {
        let base = std::env::temp_dir().join("rwc_paths_test/nested/deep");
        let _ = fs::remove_dir_all(std::env::temp_dir().join("rwc_paths_test"));

        let file = base.join("stats.db");
        ensure_parent_dir(file.to_str().unwrap()).unwrap();
        assert!(base.is_dir());

        let _ = fs::remove_dir_all(std::env::temp_dir().join("rwc_paths_test"));
    }
}
