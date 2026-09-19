//! Tauri 桌面端 —— 命令层
//!
//! 只做三件事：探测系统语言 / 查询仪表盘数据 / 重新处理日志。
//! 所有统计逻辑都复用 `rime-word-counter` 核心库，保证与 CLI 输出一致。

use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use tauri::State;

use rime_word_counter::{db, i18n, log_processor, paths, stats, GroupBy};

/// 应用运行期状态：日志与数据库路径（可由命令行参数覆盖）。
struct AppState {
    log_path: String,
    db_path: String,
}

/// 前端启动时需要的环境信息。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Bootstrap {
    /// 探测到的系统语言（`zh-CN` / `zh-TW` / `en`），与前端 strings.ts 的 key 一致
    language: &'static str,
    /// 当前使用的日志路径（供界面展示）
    log_path: String,
    /// 当前使用的数据库路径（供界面展示）
    db_path: String,
    /// 应用版本号
    version: &'static str,
}

/// 仪表盘查询参数。
///
/// `start` / `end` 留空表示使用数据库中的完整日期范围；
/// `group_by` 为 `day` / `month` / `year`，未知值回退到按日。
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct DashboardQuery {
    start: Option<String>,
    end: Option<String>,
    group_by: Option<String>,
}

/// 「重新处理日志」的返回：处理报告 + 最新仪表盘数据。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReprocessOutcome {
    report: log_processor::ProcessReport,
    dashboard: stats::Dashboard,
}

/// 把 anyhow 错误转成前端可读的字符串（带上下文链）。
fn to_message(error: impl std::fmt::Display) -> String {
    format!("{error:#}")
}

/// 按查询参数计算仪表盘数据；未指定范围时回退到数据库全量范围。
fn build_dashboard(db_path: &str, query: &DashboardQuery) -> Result<stats::Dashboard> {
    let conn = db::init_db(db_path)?;
    let (data_start, data_end) = db::query_date_range(&conn)?;

    let start = query
        .start
        .as_deref()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or(&data_start);
    let end = query
        .end
        .as_deref()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or(&data_end);
    let group_by = GroupBy::from_code(query.group_by.as_deref().unwrap_or("day"));

    stats::compute_dashboard(&conn, start, end, group_by)
}

#[tauri::command]
fn get_bootstrap(state: State<'_, AppState>) -> Bootstrap {
    Bootstrap {
        language: i18n::detect_language().code(),
        log_path: state.log_path.clone(),
        db_path: state.db_path.clone(),
        version: env!("CARGO_PKG_VERSION"),
    }
}

#[tauri::command]
fn get_dashboard(
    state: State<'_, AppState>,
    query: DashboardQuery,
) -> Result<stats::Dashboard, String> {
    build_dashboard(&state.db_path, &query).map_err(to_message)
}

#[tauri::command]
fn reprocess(
    state: State<'_, AppState>,
    query: DashboardQuery,
) -> Result<ReprocessOutcome, String> {
    let report =
        log_processor::process_logs(&state.log_path, &state.db_path).map_err(to_message)?;
    let dashboard = build_dashboard(&state.db_path, &query).map_err(to_message)?;
    Ok(ReprocessOutcome { report, dashboard })
}

/// 桌面端命令行参数（全部可选，用于自定义路径）。
#[derive(Debug, Default, clap::Parser)]
#[command(name = "rime-word-counter-gui", version, about)]
struct CliOverrides {
    /// 日志文件路径（Lua 脚本写入的 CSV）
    #[arg(long)]
    log_path: Option<String>,

    /// SQLite 数据库路径
    #[arg(long)]
    db_path: Option<String>,
}

/// 启动桌面应用。
pub fn run() {
    // 参数解析失败（例如被文件管理器传入奇怪参数）时退回默认路径，不阻断启动
    let overrides = CliOverrides::try_parse().unwrap_or_default();

    tauri::Builder::default()
        .setup(|_app| {
            // 缓存目录是 Lua 脚本与数据库的共同落点，提前确保存在。
            // 创建失败不阻断启动：真正的错误会在查询/处理时带上下文报出来。
            if let Err(error) = paths::ensure_cache_dir() {
                eprintln!("[WARN] 无法创建缓存目录: {error:#}");
            }
            Ok(())
        })
        .manage(AppState {
            log_path: overrides
                .log_path
                .clone()
                .unwrap_or_else(paths::default_log_path),
            db_path: overrides
                .db_path
                .clone()
                .unwrap_or_else(paths::default_db_path),
        })
        .invoke_handler(tauri::generate_handler![
            get_bootstrap,
            get_dashboard,
            reprocess
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
