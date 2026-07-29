//! SQLite 数据库操作封装
//!
//! 负责建表、写入和查询字数统计数据。

use anyhow::{Context, Result};
use rusqlite::{params, Connection};

/// 分组粒度
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GroupBy {
    /// 按日分组（YYYY-MM-DD）
    Day,
    /// 按月分组（YYYY-MM）
    Month,
    /// 按年分组（YYYY）
    Year,
}

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

/// 查询指定日期范围内的字数数据，按指定粒度分组。
///
/// * `start_date` — 起始日期（含），格式 YYYY-MM-DD
/// * `end_date` — 结束日期（含），格式 YYYY-MM-DD
/// * `group_by` — 分组粒度
pub fn query_grouped(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
    group_by: GroupBy,
) -> Result<Vec<(String, i64)>> {
    let sql = match group_by {
        GroupBy::Day => {
            "SELECT date, word_count
             FROM daily_words
             WHERE date >= ?1 AND date <= ?2
             ORDER BY date ASC"
        }
        GroupBy::Month => {
            "SELECT substr(date, 1, 7) AS period, SUM(word_count)
             FROM daily_words
             WHERE date >= ?1 AND date <= ?2
             GROUP BY period
             ORDER BY period ASC"
        }
        GroupBy::Year => {
            "SELECT substr(date, 1, 4) AS period, SUM(word_count)
             FROM daily_words
             WHERE date >= ?1 AND date <= ?2
             GROUP BY period
             ORDER BY period ASC"
        }
    };

    let mut stmt = conn.prepare(sql).context("准备分组查询语句失败")?;
    let rows = stmt
        .query_map(params![start_date, end_date], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
            ))
        })
        .context("执行分组查询失败")?;

    let result: Vec<(String, i64)> = rows.filter_map(|r| r.ok()).collect();
    Ok(result)
}

/// 查询总字数（所有日期的累加）。
pub fn query_total_words(conn: &Connection) -> Result<i64> {
    let total: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(word_count), 0) FROM daily_words",
            [],
            |row| row.get(0),
        )
        .context("查询总字数失败")?;
    Ok(total)
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

/// 获取数据库中的最早和最晚日期。
pub fn query_date_range(conn: &Connection) -> Result<(String, String)> {
    let (start, end): (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT MIN(date), MAX(date) FROM daily_words",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .context("查询日期范围失败")?;

    let start = start.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    let end = end.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    Ok((start, end))
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

    fn seed_test_data(conn: &Connection) {
        let data = vec![
            ("2026-01-15", 100),
            ("2026-01-20", 200),
            ("2026-02-10", 300),
            ("2026-02-15", 400),
            ("2027-03-01", 500),
        ];
        for (date, count) in data {
            upsert_word_count(conn, date, count).unwrap();
        }
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

    #[test]
    fn test_query_grouped_by_day() {
        let conn = setup_test_db();
        seed_test_data(&conn);
        let data = query_grouped(&conn, "2026-01-01", "2026-12-31", GroupBy::Day).unwrap();
        // Should return all 2026 entries in day granularity
        assert_eq!(data.len(), 4);
        assert_eq!(data[0], ("2026-01-15".to_string(), 100));
        assert_eq!(data[3], ("2026-02-15".to_string(), 400));
    }

    #[test]
    fn test_query_grouped_by_month() {
        let conn = setup_test_db();
        seed_test_data(&conn);
        let data = query_grouped(&conn, "2026-01-01", "2026-12-31", GroupBy::Month).unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0], ("2026-01".to_string(), 300));  // 100 + 200
        assert_eq!(data[1], ("2026-02".to_string(), 700));  // 300 + 400
    }

    #[test]
    fn test_query_grouped_by_year() {
        let conn = setup_test_db();
        seed_test_data(&conn);
        let data = query_grouped(&conn, "2026-01-01", "2027-12-31", GroupBy::Year).unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0], ("2026".to_string(), 1000));
        assert_eq!(data[1], ("2027".to_string(), 500));
    }

    #[test]
    fn test_query_total_words() {
        let conn = setup_test_db();
        seed_test_data(&conn);
        let total = query_total_words(&conn).unwrap();
        assert_eq!(total, 1500);
    }

    #[test]
    fn test_query_date_range() {
        let conn = setup_test_db();
        seed_test_data(&conn);
        let (start, end) = query_date_range(&conn).unwrap();
        assert_eq!(start, "2026-01-15");
        assert_eq!(end, "2027-03-01");
    }
}
