//! Rime 字数统计工具 — 入口
//!
//! 命令行参数解析与模块编排。

mod db;
mod log_processor;
mod visualizer;

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

/// Rime 输入法字数统计与可视化工具
///
/// 处理 Lua 脚本生成的 CSV 日志，按日期汇总字数存入 SQLite 数据库，
/// 并生成每日字数的可视化图表。
#[derive(Parser, Debug)]
#[command(
    name = "rime-word-counter",
    version,
    about,
    long_about = None
)]
struct Cli {
    /// 仅处理日志文件（处理 → 汇总 → 清空），不生成图表
    #[arg(long)]
    process: bool,

    /// 仅生成图表（从已有数据库读取），不处理日志
    #[arg(long)]
    visualize: bool,

    /// 日志文件路径（Lua 脚本写入的 CSV）
    #[arg(long, default_value = "")]
    log_path: String,

    /// SQLite 数据库路径
    #[arg(long, default_value = "")]
    db_path: String,

    /// 图表输出路径（PNG 文件）
    #[arg(long, default_value = "word_stats.png")]
    output: String,

    /// 图表涵盖的天数
    #[arg(long, default_value_t = 30)]
    days: i64,
}

/// 获取默认的缓存目录路径。
fn get_cache_dir() -> PathBuf {
    let base = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("rime-word-counter")
}

/// 获取默认的日志文件路径。
fn default_log_path() -> String {
    get_cache_dir()
        .join("rime_word.log")
        .to_string_lossy()
        .to_string()
}

/// 获取默认的数据库路径。
fn default_db_path() -> String {
    get_cache_dir()
        .join("rime_stats.db")
        .to_string_lossy()
        .to_string()
}

/// 确保缓存目录存在。
fn ensure_cache_dir(cache_dir: &PathBuf) -> Result<()> {
    fs::create_dir_all(cache_dir)
        .with_context(|| format!("无法创建缓存目录: {}", cache_dir.display()))
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // 解析路径（允许通过命令行参数覆盖默认值）
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

    // 确保缓存目录存在
    let cache_dir = get_cache_dir();
    ensure_cache_dir(&cache_dir)?;

    // 判断执行模式
    let do_process = cli.process || !cli.visualize;  // 只设 --visualize 时不处理日志
    let do_visualize = cli.visualize || !cli.process; // 只设 --process 时不生成图表

    if do_process {
        println!("[INFO] 开始处理日志文件...");
        println!("[INFO]   日志路径: {log_path}");
        println!("[INFO]   数据库路径: {db_path}");
        log_processor::process_logs(&log_path, &db_path)?;
        println!("[INFO] 日志处理完成");
    }

    if do_visualize {
        println!("[INFO] 开始生成图表...");
        println!("[INFO]   数据库路径: {db_path}");
        println!("[INFO]   输出路径: {}", cli.output);
        println!("[INFO]   涵盖天数: {}", cli.days);
        visualizer::generate_chart(&db_path, &cli.output, cli.days)?;
        println!("[INFO] 图表生成完成");
    }

    Ok(())
}
