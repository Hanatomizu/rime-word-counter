//! 聚合统计模块
//!
//! 把数据库中的原始「按日」记录，转换成前端直接可用的仪表盘数据：
//! 分组后的趋势点 + 概览统计（总量/日均/峰值/覆盖天数）。
//!
//! 这里是 CLI 与 Tauri 桌面端唯一的数据出口，保证两端算法一致。

use anyhow::{Context, Result};
use chrono::NaiveDate;
use rusqlite::Connection;
use serde::Serialize;

use crate::db::{self, GroupBy};

/// 单个趋势数据点（已按分组粒度聚合）。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Point {
    /// 分组标签：`2026-07-29` / `2026-07` / `2026`
    pub label: String,
    /// 该分组内的总字数
    pub count: i64,
}

/// 概览统计。
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    /// 数据库全量累计字数（不受筛选影响）
    pub total_all: i64,
    /// 当前筛选范围内的累计字数
    pub total_range: i64,
    /// 筛选范围内的「有记录天数」
    pub days: usize,
    /// 趋势图中的数据点数量（随分组粒度变化）
    pub buckets: usize,
    /// 日均字数（total_range / days，保留一位小数）
    pub average: f64,
    /// 当前分组下的最大值
    pub max: i64,
    /// 当前分组下的最小值
    pub min: i64,
}

/// 前端一次性渲染所需的全部数据。
///
/// 序列化为 camelCase，与 `ui/src/types.ts` 一一对应。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dashboard {
    /// 趋势数据点，按标签升序
    pub points: Vec<Point>,
    /// 概览统计
    pub summary: Summary,
    /// 数据库中的最早日期（无数据时为今天）
    pub data_start: String,
    /// 数据库中的最晚日期（无数据时为今天）
    pub data_end: String,
    /// 本次查询使用的起始日期
    pub range_start: String,
    /// 本次查询使用的结束日期
    pub range_end: String,
    /// 本次查询使用的分组粒度（`day` / `month` / `year`）
    pub group_by: GroupBy,
}

/// 计算仪表盘数据。
///
/// * `start` / `end` —— 闭区间日期，格式 `YYYY-MM-DD`
/// * `group_by` —— 日 / 月 / 年 分组粒度
///
/// 日期格式非法时返回错误（而不是静默返回空数据），
/// 这样前端可以把「输入还没写完」和「真的没数据」区分开。
pub fn compute_dashboard(
    conn: &Connection,
    start: &str,
    end: &str,
    group_by: GroupBy,
) -> Result<Dashboard> {
    let start_date = parse_date(start, "开始日期")?;
    let end_date = parse_date(end, "结束日期")?;

    if start_date > end_date {
        anyhow::bail!("开始日期晚于结束日期: {start} > {end}");
    }

    let rows = db::query_grouped(conn, start, end, group_by)
        .with_context(|| format!("查询 {start} ~ {end} 的分组数据失败"))?;

    let points: Vec<Point> = rows
        .into_iter()
        .map(|(label, count)| Point { label, count })
        .collect();

    let total_range: i64 = points.iter().map(|p| p.count).sum();
    let days = db::count_days_in_range(conn, start, end)?;
    let total_all = db::query_total_words(conn)?;
    let (data_start, data_end) = db::query_date_range(conn)?;

    let max = points.iter().map(|p| p.count).max().unwrap_or(0);
    let min = points.iter().map(|p| p.count).min().unwrap_or(0);
    let average = if days > 0 {
        // 保留一位小数，避免前端再处理精度
        (total_range as f64 / days as f64 * 10.0).round() / 10.0
    } else {
        0.0
    };

    Ok(Dashboard {
        summary: Summary {
            total_all,
            total_range,
            days,
            buckets: points.len(),
            average,
            max,
            min,
        },
        points,
        data_start,
        data_end,
        range_start: start.to_string(),
        range_end: end.to_string(),
        group_by,
    })
}

/// 校验并解析 `YYYY-MM-DD` 日期。
fn parse_date(value: &str, field: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .with_context(|| format!("{field} 格式应为 YYYY-MM-DD，实际为 {value:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE daily_words (
                date       TEXT PRIMARY KEY,
                word_count INTEGER NOT NULL DEFAULT 0
            );",
        )
        .unwrap();
        for (date, count) in [
            ("2026-01-15", 100),
            ("2026-01-20", 200),
            ("2026-02-10", 300),
            ("2026-02-15", 400),
            ("2027-03-01", 500),
        ] {
            db::upsert_word_count(&conn, date, count).unwrap();
        }
        conn
    }

    #[test]
    fn test_dashboard_by_day() {
        let conn = setup();
        let d = compute_dashboard(&conn, "2026-01-01", "2026-12-31", GroupBy::Day).unwrap();

        assert_eq!(d.points.len(), 4);
        assert_eq!(
            d.points[0],
            Point {
                label: "2026-01-15".into(),
                count: 100
            }
        );
        assert_eq!(d.summary.total_range, 1000);
        assert_eq!(d.summary.total_all, 1500);
        assert_eq!(d.summary.days, 4);
        assert_eq!(d.summary.buckets, 4);
        assert_eq!(d.summary.average, 250.0);
        assert_eq!(d.summary.max, 400);
        assert_eq!(d.summary.min, 100);
        assert_eq!(d.data_start, "2026-01-15");
        assert_eq!(d.data_end, "2027-03-01");
    }

    #[test]
    fn test_dashboard_by_month_aggregates_and_averages_per_day() {
        let conn = setup();
        let d = compute_dashboard(&conn, "2026-01-01", "2026-12-31", GroupBy::Month).unwrap();

        assert_eq!(d.points.len(), 2);
        assert_eq!(
            d.points[0],
            Point {
                label: "2026-01".into(),
                count: 300
            }
        );
        assert_eq!(
            d.points[1],
            Point {
                label: "2026-02".into(),
                count: 700
            }
        );
        // 日均按「有记录的天数」计算，而不是按分组数
        assert_eq!(d.summary.days, 4);
        assert_eq!(d.summary.buckets, 2);
        assert_eq!(d.summary.average, 250.0);
        assert_eq!(d.summary.max, 700);
        assert_eq!(d.summary.min, 300);
    }

    #[test]
    fn test_dashboard_by_year() {
        let conn = setup();
        let d = compute_dashboard(&conn, "2026-01-01", "2027-12-31", GroupBy::Year).unwrap();
        assert_eq!(d.points.len(), 2);
        assert_eq!(
            d.points[0],
            Point {
                label: "2026".into(),
                count: 1000
            }
        );
        assert_eq!(
            d.points[1],
            Point {
                label: "2027".into(),
                count: 500
            }
        );
        assert_eq!(d.summary.total_range, 1500);
    }

    #[test]
    fn test_dashboard_empty_range_is_zeroed_not_error() {
        let conn = setup();
        let d = compute_dashboard(&conn, "2025-01-01", "2025-12-31", GroupBy::Day).unwrap();
        assert!(d.points.is_empty());
        assert_eq!(d.summary.total_range, 0);
        assert_eq!(d.summary.days, 0);
        assert_eq!(d.summary.average, 0.0);
        assert_eq!(d.summary.max, 0);
        assert_eq!(d.summary.min, 0);
        // 全量数据依然可读，供「全部」快捷按钮使用
        assert_eq!(d.summary.total_all, 1500);
    }

    #[test]
    fn test_dashboard_invalid_date_errors() {
        let conn = setup();
        assert!(compute_dashboard(&conn, "2026-13-01", "2026-12-31", GroupBy::Day).is_err());
        assert!(compute_dashboard(&conn, "2026-01-01", "not-a-date", GroupBy::Day).is_err());
    }

    #[test]
    fn test_dashboard_reversed_range_errors() {
        let conn = setup();
        assert!(compute_dashboard(&conn, "2026-12-31", "2026-01-01", GroupBy::Day).is_err());
    }
}
