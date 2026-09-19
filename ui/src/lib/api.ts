/**
 * 后端调用层。
 *
 * 在 Tauri 里走 `invoke`；直接用浏览器打开 `npm run dev`（没有 Tauri 全局对象）时
 * 退回本地生成的演示数据，方便纯前端调试 UI。
 * 演示模式会在界面上显示「演示数据」标记，避免和真实统计混淆。
 */

import type { Bootstrap, Dashboard, GroupBy, Point, ReprocessOutcome } from '../types';
import { addDays, isValidISODate, todayISO } from './date';

/** 是否运行在 Tauri 容器里 */
export const IS_TAURI: boolean =
  typeof window !== 'undefined' &&
  ('__TAURI_INTERNALS__' in window || '__TAURI__' in window);

/** 是否处于浏览器演示模式 */
export const IS_DEMO = !IS_TAURI;

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(command, args);
}

export interface DashboardQuery {
  start?: string;
  end?: string;
  groupBy?: GroupBy;
}

/** 启动信息：系统语言、路径、版本 */
export async function getBootstrap(): Promise<Bootstrap> {
  if (IS_DEMO) {
    return {
      language: 'zh-CN',
      logPath: '~/.cache/rime-word-counter/rime_word.log',
      dbPath: '~/.cache/rime-word-counter/rime_stats.db',
      version: '1.0.0-r2 (demo)',
    };
  }
  return call<Bootstrap>('get_bootstrap');
}

/** 查询仪表盘数据；`start`/`end` 留空表示数据库全量范围 */
export async function getDashboard(query: DashboardQuery = {}): Promise<Dashboard> {
  if (IS_DEMO) return mockDashboard(query);
  return call<Dashboard>('get_dashboard', { query });
}

/** 重新处理日志，并返回处理报告 + 最新数据 */
export async function reprocess(query: DashboardQuery = {}): Promise<ReprocessOutcome> {
  if (IS_DEMO) {
    await new Promise((resolve) => setTimeout(resolve, 420));
    return {
      report: { linesRead: 37, parseErrors: 0, datesUpdated: 6 },
      dashboard: mockDashboard(query),
    };
  }
  return call<ReprocessOutcome>('reprocess', { query });
}

// ---------------------------------------------------------------------------
// 演示模式用的假数据（仅在没有 Tauri 后端时使用）
// ---------------------------------------------------------------------------

const MOCK_DAYS = 210;

/** 线性同余伪随机：固定种子，保证每次预览的数据一致，便于比对视觉改动 */
function createMockDaily(): Point[] {
  let seed = 20260729;
  const random = () => {
    seed = (seed * 1103515245 + 12345) % 2147483648;
    return seed / 2147483648;
  };

  const end = todayISO();
  const points: Point[] = [];

  for (let offset = MOCK_DAYS - 1; offset >= 0; offset -= 1) {
    const label = addDays(end, -offset);
    const weekday = new Date(`${label}T00:00:00`).getDay();
    const weekend = weekday === 0 || weekday === 6;

    const base = weekend ? 900 : 2600;
    const noise = random() * 2200;
    const spike = random() > 0.94 ? random() * 5200 : 0;

    points.push({ label, count: Math.round(base + noise + spike) });
  }
  return points;
}

function groupPoints(daily: Point[], groupBy: GroupBy): Point[] {
  if (groupBy === 'day') return daily;

  const buckets = new Map<string, number>();
  for (const point of daily) {
    const key = groupBy === 'month' ? point.label.slice(0, 7) : point.label.slice(0, 4);
    buckets.set(key, (buckets.get(key) ?? 0) + point.count);
  }

  return [...buckets.entries()]
    .sort((a, b) => a[0].localeCompare(b[0]))
    .map(([label, count]) => ({ label, count }));
}

function mockDashboard(query: DashboardQuery): Dashboard {
  const daily = createMockDaily();
  const dataStart = daily[0]?.label ?? todayISO();
  const dataEnd = daily[daily.length - 1]?.label ?? todayISO();

  const start = query.start?.trim() || dataStart;
  const end = query.end?.trim() || dataEnd;
  const groupBy = query.groupBy ?? 'day';

  if (!isValidISODate(start) || !isValidISODate(end)) {
    throw new Error('日期格式应为 YYYY-MM-DD');
  }
  if (start > end) {
    throw new Error(`开始日期晚于结束日期: ${start} > ${end}`);
  }

  const inRange = daily.filter((point) => point.label >= start && point.label <= end);
  const points = groupPoints(inRange, groupBy);

  const totalRange = points.reduce((sum, point) => sum + point.count, 0);
  const counts = points.map((point) => point.count);

  return {
    points,
    summary: {
      totalAll: daily.reduce((sum, point) => sum + point.count, 0),
      totalRange,
      days: inRange.length,
      buckets: points.length,
      average: inRange.length > 0 ? Math.round((totalRange / inRange.length) * 10) / 10 : 0,
      max: counts.length > 0 ? Math.max(...counts) : 0,
      min: counts.length > 0 ? Math.min(...counts) : 0,
    },
    dataStart,
    dataEnd,
    rangeStart: start,
    rangeEnd: end,
    groupBy,
  };
}
