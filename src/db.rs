//! SQLite 数据库操作封装
//!
//! 负责建表、写入和查询字数统计数据。

use anyhow::{Context, Result};
use rusqlite::{params, Connection};

/// 初始化数据库，创建 `daily_words` 表（如果不存在）。
pub fn init_db(db_path: &str) -> Result<Connection> {
    let conn = Connection::open(db_path)
        .with_context(|| format!("无法打开数据库: {db_path}"))?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS daily_words (
            date       TEXT PRIMARY KEY,  -- 日期，格式 YYYY-MM-DD
            word_count INTEGER NOT NULL DEFAULT 0  -- 当日总字数
        );",
    )
    .context("创建 daily_words 表失败")?;

    Ok(conn)
}

/// 插入或累加指定日期的字数。
///
/// 如果该日期已有记录，则将 `count` 累加到 `word_count`；
/// 如果没有则插入新记录。
pub fn upsert_word_count(conn: &Connection, date: &str, count: i64) -> Result<()> {
    conn.execute(
        "INSERT INTO daily_words (date, word_count)
         VALUES (?1, ?2)
         ON CONFLICT(date) DO UPDATE SET
             word_count = word_count + ?2",
        params![date, count],
    )
    .context("更新字数记录失败")?;
    Ok(())
}

/// 查询最近 N 天的字数数据，按日期升序返回。
///
/// 返回 `Vec<(date, word_count)>`，不足 N 天时返回全部可用数据。
pub fn query_recent_days(conn: &Connection, days: i64) -> Result<Vec<(String, i64)>> {
    let mut stmt = conn
        .prepare(
            "SELECT date, word_count
             FROM daily_words
             ORDER BY date DESC
             LIMIT ?1",
        )
        .context("准备查询语句失败")?;

    let rows = stmt
        .query_map(params![days], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
            ))
        })
        .context("查询字数数据失败")?;

    let mut result: Vec<(String, i64)> = rows
        .filter_map(|r| r.ok())
        .collect();

    // 按日期升序排列（图表从左到右显示）
    result.reverse();
    Ok(result)
}

/// 查询全部字数数据，按日期升序返回。
#[allow(dead_code)]
pub fn query_all(conn: &Connection) -> Result<Vec<(String, i64)>> {
    let mut stmt = conn
        .prepare(
            "SELECT date, word_count
             FROM daily_words
             ORDER BY date ASC",
        )
        .context("准备查询语句失败")?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
            ))
        })
        .context("查询全部字数数据失败")?;

    let result: Vec<(String, i64)> = rows.filter_map(|r| r.ok()).collect();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS daily_words (
                date       TEXT PRIMARY KEY,
                word_count INTEGER NOT NULL DEFAULT 0
            );",
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_upsert_new_record() {
        let conn = setup_test_db();
        upsert_word_count(&conn, "2026-07-29", 100).unwrap();

        let data = query_all(&conn).unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0], ("2026-07-29".to_string(), 100));
    }

    #[test]
    fn test_upsert_accumulate() {
        let conn = setup_test_db();
        upsert_word_count(&conn, "2026-07-29", 100).unwrap();
        upsert_word_count(&conn, "2026-07-29", 50).unwrap();

        let data = query_all(&conn).unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0], ("2026-07-29".to_string(), 150));
    }

    #[test]
    fn test_query_recent_days() {
        let conn = setup_test_db();
        upsert_word_count(&conn, "2026-07-28", 200).unwrap();
        upsert_word_count(&conn, "2026-07-29", 100).unwrap();
        upsert_word_count(&conn, "2026-07-30", 300).unwrap();

        // 查最近 2 天 → 29, 30 号
        let data = query_recent_days(&conn, 2).unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0].0, "2026-07-29");
        assert_eq!(data[1].0, "2026-07-30");
    }

    #[test]
    fn test_query_all_ordered() {
        let conn = setup_test_db();
        upsert_word_count(&conn, "2026-07-30", 300).unwrap();
        upsert_word_count(&conn, "2026-07-28", 200).unwrap();
        upsert_word_count(&conn, "2026-07-29", 100).unwrap();

        let data = query_all(&conn).unwrap();
        assert_eq!(data.len(), 3);
        assert_eq!(data[0].0, "2026-07-28");
        assert_eq!(data[1].0, "2026-07-29");
        assert_eq!(data[2].0, "2026-07-30");
    }
}
