//! Rime 字数统计 —— 命令行入口
//!
//! 面向定时任务与终端排查：
//!
//! * `--process` —— 处理日志（读取 → 汇总 → 写入 SQLite → 清空日志），cron 用
//! * `--stats`   —— 在终端打印统计摘要，不依赖图形界面
//!
//! 图形界面由独立的 Tauri 应用提供，见 `src-tauri/`（二进制名
//! `rime-word-counter-gui`，也可用 `npm run tauri dev` 启动）。

use anyhow::Result;
use clap::Parser;

use rime_word_counter::{db, log_processor, paths, stats, GroupBy};

/// Rime 输入法字数统计与可视化工具（命令行）
#[derive(Parser, Debug)]
#[command(name = "rime-word-counter", version, about, long_about = None)]
struct Cli {
    /// 处理日志文件（读取 → 汇总 → 清空），适用于定时任务
    #[arg(long)]
    process: bool,

    /// 在终端打印统计摘要
    #[arg(long)]
    stats: bool,

    /// 日志文件路径（Lua 脚本写入的 CSV）
    #[arg(long, default_value = "")]
    log_path: String,

    /// SQLite 数据库路径
    #[arg(long, default_value = "")]
    db_path: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let log_path = if cli.log_path.is_empty() {
        paths::default_log_path()
    } else {
        cli.log_path.clone()
    };
    let db_path = if cli.db_path.is_empty() {
        paths::default_db_path()
    } else {
        cli.db_path.clone()
    };

    if cli.process {
        println!("[INFO] 开始处理日志文件...");
        println!("[INFO]   日志路径: {log_path}");
        println!("[INFO]   数据库路径: {db_path}");
        log_processor::process_logs(&log_path, &db_path)?;
        println!("[INFO] 处理完成");
        return Ok(());
    }

    if cli.stats {
        return print_stats(&db_path);
    }

    print_usage_hint(&log_path, &db_path);
    Ok(())
}

/// 在终端打印统计摘要。
fn print_stats(db_path: &str) -> Result<()> {
    let conn = db::init_db(db_path)?;
    let (start, end) = db::query_date_range(&conn)?;
    let dashboard = stats::compute_dashboard(&conn, &start, &end, GroupBy::Day)?;
    let s = &dashboard.summary;

    println!("Rime 字数统计");
    println!("  数据库      {db_path}");
    println!(
        "  日期范围    {} ~ {}",
        dashboard.data_start, dashboard.data_end
    );
    if s.days == 0 {
        println!("  暂无数据，请先用 Rime 输入法打字，或检查 --log-path 是否正确");
        return Ok(());
    }
    println!("  总字数      {} 字", format_count(s.total_all));
    println!("  有记录天数  {} 天", s.days);
    println!(
        "  日均        {} 字/日",
        format_count(s.average.round() as i64)
    );
    println!("  单日最高    {} 字", format_count(s.max));
    println!("  单日最低    {} 字", format_count(s.min));
    Ok(())
}

/// 没有任何模式参数时，说明两个前端各自怎么启动。
fn print_usage_hint(log_path: &str, db_path: &str) {
    println!("Rime 字数统计 —— 命令行");
    println!();
    println!("  图形界面      npm run tauri dev          (开发)");
    println!("                npm run tauri build        (打包桌面应用)");
    println!("  处理日志      rime-word-counter --process");
    println!("  查看统计      rime-word-counter --stats");
    println!();
    println!("  日志路径      {log_path}");
    println!("  数据库路径    {db_path}");
    println!();
    println!("  用 --help 查看全部参数。");
}

/// 格式化数字（例：12345 → "12,345"）。
///
/// 这里刻意用 `% 3` 而不是 `is_multiple_of`：后者需要 Rust 1.87，
/// 而本项目的 MSRV 跟随 Tauri 模板保持在 1.77。
#[allow(clippy::manual_is_multiple_of)]
fn format_count(n: i64) -> String {
    let negative = n < 0;
    let digits = n.abs().to_string();
    let mut out = String::new();

    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }

    if negative {
        format!("-{out}")
    } else {
        out
    }
}
