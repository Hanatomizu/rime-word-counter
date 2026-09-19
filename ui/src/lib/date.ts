/** 日期工具：后端只认 `YYYY-MM-DD`，这里统一按本地时区计算，避免时区偏移 */

import type { GroupBy } from '../types';

const ISO_PATTERN = /^\d{4}-\d{2}-\d{2}$/;

/** 把 Date 转成 `YYYY-MM-DD`（本地时区，不走 toISOString 以免被 UTC 拉偏） */
export function toISODate(date: Date): string {
  const year = date.getFullYear();
  const month = `${date.getMonth() + 1}`.padStart(2, '0');
  const day = `${date.getDate()}`.padStart(2, '0');
  return `${year}-${month}-${day}`;
}

/** 今天 */
export function todayISO(): string {
  return toISODate(new Date());
}

/** 在 ISO 日期上加减天数 */
export function addDays(iso: string, days: number): string {
  const date = new Date(`${iso}T00:00:00`);
  date.setDate(date.getDate() + days);
  return toISODate(date);
}

/** 最近 N 天（含今天）的区间 */
export function lastDays(days: number): { start: string; end: string } {
  const end = todayISO();
  return { start: addDays(end, -(days - 1)), end };
}

/** 校验是否为合法且真实存在的 `YYYY-MM-DD` */
export function isValidISODate(value: string): boolean {
  if (!ISO_PATTERN.test(value)) return false;
  const date = new Date(`${value}T00:00:00`);
  return !Number.isNaN(date.getTime()) && toISODate(date) === value;
}

/** 趋势图 X 轴标签：按分组粒度截断，避免坐标轴被年份占满 */
export function shortLabel(label: string, groupBy: GroupBy): string {
  if (groupBy === 'day') return label.slice(5); // 2026-07-29 → 07-29
  if (groupBy === 'month') return label.replace('-', '/'); // 2026-07 → 2026/07
  return label;
}

/** 表格里展示的范围文案 */
export function rangeText(start: string, end: string): string {
  return start === end ? start : `${start} → ${end}`;
}

/** 侧栏的快捷时间范围 */
export type PresetId = 'last7' | 'last30' | 'lastYear' | 'all';

/** 全部快捷范围，顺序即界面顺序 */
export const PRESET_IDS: PresetId[] = ['last7', 'last30', 'lastYear', 'all'];

/**
 * 计算快捷范围。
 *
 * 「全部」用数据库里的真实起止日期（而不是今天），这样能覆盖所有历史数据。
 */
export function presetRange(
  id: PresetId,
  dataStart: string,
  dataEnd: string,
): { start: string; end: string } {
  if (id === 'all') return { start: dataStart, end: dataEnd };

  const days = id === 'last7' ? 7 : id === 'last30' ? 30 : 365;
  return lastDays(days);
}

