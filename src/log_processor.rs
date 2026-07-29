//! 日志处理模块
//!
//! 读取 Lua 脚本生成的 CSV 日志文件，按日期汇总字数，
//! 更新到 SQLite 数据库后清空日志文件。

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader};

use anyhow::{Context, Result};
use chrono::NaiveDate;

use crate::db;

/// 处理日志文件：读取 → 按日期分组累加 → 更新数据库 → 清空文件。
///
/// 如果日志文件不存在，直接返回成功（可能是首次运行）。
pub fn process_logs(log_path: &str, db_path: &str) -> Result<()> {
    let log_file = match OpenOptions::new().read(true).write(true).open(log_path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("[INFO] 日志文件不存在，跳过处理: {log_path}");
            return Ok(());
        }
        Err(e) => return Err(e).context(format!("无法打开日志文件: {log_path}")),
    };

    // 读取所有行
    let reader = BufReader::new(&log_file);
    let lines: Vec<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .filter(|l| !l.trim().is_empty())
        .collect();

    if lines.is_empty() {
        println!("[INFO] 日志文件为空，无需处理");
        return Ok(());
    }

    println!("[INFO] 读取到 {} 行日志记录", lines.len());

    // 解析并按日期分组累加
    let mut daily_totals: HashMap<String, i64> = HashMap::new();
    let mut parse_errors = 0;

    for line in &lines {
        match parse_log_line(line) {
            Ok((date, count)) => {
                *daily_totals.entry(date).or_insert(0) += count;
            }
            Err(e) => {
                eprintln!("[WARN] 解析行失败 (已跳过): {e} → 内容: {line}");
                parse_errors += 1;
            }
        }
    }

    if parse_errors > 0 {
        println!("[INFO] {parse_errors} 行解析失败，已跳过");
    }

    // 更新数据库（使用事务）
    let conn = db::init_db(db_path)?;

    // 开始事务
    conn.execute_batch("BEGIN TRANSACTION")
        .context("开始数据库事务失败")?;

    let batch_result = {
        let mut entries: Vec<(&String, &i64)> = daily_totals.iter().collect();
        entries.sort_by_key(|(date, _)| *date); // 按日期排序输出

        for (date, &count) in &entries {
            println!("[INFO]    {date}: +{count} 字");
            db::upsert_word_count(&conn, date, count)
                .with_context(|| format!("更新 {date} 的记录失败"))?;
        }

        entries.len()
    };

    // 提交事务
    conn.execute_batch("COMMIT")
        .context("提交数据库事务失败")?;

    println!("[INFO] 成功更新 {batch_result} 条日期记录");

    // 清空日志文件（截断为 0 字节）
    log_file.set_len(0).context("清空日志文件失败")?;
    println!("[INFO] 日志文件已清空");

    Ok(())
}

/// 解析一行 CSV 日志，返回 `(日期字符串, 字数)`。
///
/// 预期格式：`YYYY-MM-DD,count`（例如 `2026-07-29,5`）
fn parse_log_line(line: &str) -> Result<(String, i64)> {
    let line = line.trim();
    let comma_pos = line
        .rfind(',')
        .ok_or_else(|| anyhow::anyhow!("缺少逗号分隔符"))?;

    let date_str = line[..comma_pos].trim();
    let count_str = line[comma_pos + 1..].trim();

    // 验证日期格式
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .with_context(|| format!("无效日期格式: {date_str}"))?;

    let count: i64 = count_str
        .parse()
        .with_context(|| format!("无效数字: {count_str}"))?;

    Ok((date_str.to_string(), count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;

    #[test]
    fn test_parse_valid_line() {
        let (date, count) = parse_log_line("2026-07-29,5").unwrap();
        assert_eq!(date, "2026-07-29");
        assert_eq!(count, 5);
    }

    #[test]
    fn test_parse_line_with_spaces() {
        let (date, count) = parse_log_line("  2026-07-29  ,  42  ").unwrap();
        assert_eq!(date, "2026-07-29");
        assert_eq!(count, 42);
    }

    #[test]
    fn test_parse_invalid_date() {
        assert!(parse_log_line("2026-13-01,5").is_err());
    }

    #[test]
    fn test_parse_missing_comma() {
        assert!(parse_log_line("20260729").is_err());
    }

    #[test]
    fn test_parse_non_numeric_count() {
        assert!(parse_log_line("2026-07-29,abc").is_err());
    }

    #[test]
    fn test_process_logs_integration() {
        // 创建临时日志文件
        let tmp_dir = std::env::temp_dir();
        let log_path = tmp_dir.join("test_rime_word.log");
        let db_path = tmp_dir.join("test_rime_stats.db");

        // 写入测试数据
        {
            let mut file = File::create(&log_path).unwrap();
            writeln!(file, "2026-07-28,100").unwrap();
            writeln!(file, "2026-07-29,200").unwrap();
            writeln!(file, "2026-07-29,50").unwrap();  // 同一天追加
        }

        // 处理日志
        process_logs(
            log_path.to_str().unwrap(),
            db_path.to_str().unwrap(),
        )
        .unwrap();

        // 验证数据库
        let conn = db::init_db(db_path.to_str().unwrap()).unwrap();
        let data = db::query_all(&conn).unwrap();

        assert_eq!(data.len(), 2);
        assert_eq!(data[0], ("2026-07-28".to_string(), 100));
        assert_eq!(data[1], ("2026-07-29".to_string(), 250)); // 200 + 50

        // 验证日志文件已被清空
        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.is_empty());

        // 清理临时文件
        let _ = fs::remove_file(&log_path);
        let _ = fs::remove_file(&db_path);
    }
}
