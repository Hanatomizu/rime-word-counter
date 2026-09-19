//! Rime 字数统计 —— 核心库
//!
//! 提供日志采集（[`log_processor`]）、SQLite 存储（[`db`]）、
//! 聚合统计（[`stats`]）、路径解析（[`paths`]）与语言探测（[`i18n`]）。
//!
//! 该库同时被两个前端复用：
//!
//! * `rime-word-counter` —— 命令行（`--process` / `--stats`，用于 cron）
//! * `rime-word-counter-gui` —— Tauri 桌面应用（见 `src-tauri/`）

pub mod db;
pub mod i18n;
pub mod log_processor;
pub mod paths;
pub mod stats;

pub use db::GroupBy;
pub use i18n::Language;
pub use stats::{compute_dashboard, Dashboard, Point, Summary};
