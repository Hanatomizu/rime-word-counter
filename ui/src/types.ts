/**
 * 与 Rust 端一一对应的数据结构。
 *
 * Rust 侧的 DTO 统一用 `#[serde(rename_all = "camelCase")]` 序列化，
 * 所以这里全部是 camelCase；改动 Rust 结构体时请同步本文件。
 */

/** 分组粒度 */
export type GroupBy = 'day' | 'month' | 'year';

/** 界面语言 */
export type Lang = 'zh-CN' | 'zh-TW' | 'en';

/** `get_bootstrap` 的返回：启动时的环境信息 */
export interface Bootstrap {
  language: Lang;
  logPath: string;
  dbPath: string;
  version: string;
}

/** 趋势图上的一个数据点 */
export interface Point {
  label: string;
  count: number;
}

/** 概览统计（对应 Rust `stats::Summary`） */
export interface Summary {
  /** 数据库全量累计 */
  totalAll: number;
  /** 当前筛选范围内累计 */
  totalRange: number;
  /** 范围内有记录的天数 */
  days: number;
  /** 趋势图数据点数量 */
  buckets: number;
  /** 日均字数 */
  average: number;
  /** 当前分组下的最大值 */
  max: number;
  /** 当前分组下的最小值 */
  min: number;
}

/** `get_dashboard` 的返回（对应 Rust `stats::Dashboard`） */
export interface Dashboard {
  points: Point[];
  summary: Summary;
  dataStart: string;
  dataEnd: string;
  rangeStart: string;
  rangeEnd: string;
  groupBy: GroupBy;
}

/** `reprocess` 的结果：处理报告 */
export interface ProcessReport {
  linesRead: number;
  parseErrors: number;
  datesUpdated: number;
}

/** `reprocess` 的返回：处理报告 + 最新仪表盘 */
export interface ReprocessOutcome {
  report: ProcessReport;
  dashboard: Dashboard;
}
