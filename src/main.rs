//! Rime 字数统计工具 — 入口
//!
//! 支持 CLI 模式（--process）和 GUI 模式（--gui / 默认）。

mod db;
mod gui;
mod i18n;
mod log_processor;
mod visualizer;

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

/// Rime 输入法字数统计与可视化工具
///
/// 默认启动 GUI 界面。
/// 使用 --process 可在不启动 GUI 的情况下仅处理日志（适用于定时任务）。
#[derive(Parser, Debug)]
#[command(
    name = "rime-word-counter",
    version,
    about,
    long_about = None
)]
struct Cli {
    /// 仅处理日志文件（处理 → 汇总 → 清空），不启动 GUI
    #[arg(long)]
    process: bool,

    /// 强制启动 GUI 界面（默认行为）
    #[arg(long)]
    gui: bool,

    /// 日志文件路径（Lua 脚本写入的 CSV）
    #[arg(long, default_value = "")]
    log_path: String,

    /// SQLite 数据库路径
    #[arg(long, default_value = "")]
    db_path: String,
}

/// 获取默认的缓存目录路径。
pub fn get_cache_dir() -> PathBuf {
    let base = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("rime-word-counter")
}

/// 获取默认的日志文件路径。
pub fn default_log_path() -> String {
    get_cache_dir()
        .join("rime_word.log")
        .to_string_lossy()
        .to_string()
}

/// 获取默认的数据库路径。
pub fn default_db_path() -> String {
    get_cache_dir()
        .join("rime_stats.db")
        .to_string_lossy()
        .to_string()
}

/// 确保缓存目录存在。
pub fn ensure_cache_dir(cache_dir: &PathBuf) -> Result<()> {
    fs::create_dir_all(cache_dir)
        .with_context(|| format!("无法创建缓存目录: {}", cache_dir.display()))
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let log_path = if cli.log_path.is_empty() {
        default_log_path()
    } else {
        cli.log_path
    };

    let db_path = if cli.db_path.is_empty() {
        default_db_path()
    } else {
        cli.db_path
    };

    ensure_cache_dir(&get_cache_dir())?;

    // 模式判断
    // --process → 仅处理日志（用于 cron 定时任务）
    // --gui 或默认 → 启动 GUI（自动先处理日志）
    if cli.process && !cli.gui {
        // 仅处理日志模式
        println!("[INFO] 开始处理日志文件...");
        println!("[INFO]   日志路径: {log_path}");
        println!("[INFO]   数据库路径: {db_path}");
        log_processor::process_logs(&log_path, &db_path)?;
        println!("[INFO] 处理完成");
    } else {
        // GUI 模式（默认）
        gui::run_gui(&db_path)?;
    }

    Ok(())
}
