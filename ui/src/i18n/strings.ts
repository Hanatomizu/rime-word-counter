/**
 * 界面文案（简体中文 / 繁体中文 / English）。
 *
 * 以简体中文为「基准表」：`Strings` 类型由它推导，
 * 因此漏翻译某个 key 会在 `tsc` 阶段直接报错。
 */

import type { Lang } from '../types';

export type { Lang };

const zhCN = {
  appName: 'Rime 字数统计',
  appTagline: '输入法打字量追踪',

  // 分组
  groupBy: '分组',
  groupDay: '日',
  groupMonth: '月',
  groupYear: '年',

  // 时间范围
  dateRange: '时间范围',
  startDate: '开始日期',
  endDate: '结束日期',
  quickRange: '快捷选择',
  last7Days: '最近 7 天',
  last30Days: '最近 30 天',
  lastYear: '最近一年',
  allTime: '全部',

  // 数据源
  dataSource: '数据源',
  logFile: '日志',
  database: '数据库',

  // 操作
  reprocess: '重新处理',
  reprocessing: '处理中…',
  reprocessHint: '读取日志并汇入数据库',
  retry: '重试',

  // 概览指标
  totalAll: '总字数',
  totalRange: '范围内字数',
  dailyAverage: '日均',
  coveredDays: '覆盖天数',
  peak: '最高',
  trough: '最低',
  buckets: '数据点',
  unitWords: '字',
  unitDays: '天',
  perDay: '/ 天',

  // 图表
  trendTitle: '字数趋势',
  recordsTitle: '明细记录',
  chartEmpty: '当前范围内没有数据',
  chartEmptyHint: '换个时间范围，或先用 Rime 输入法打几个字',

  // 表格
  colDate: '日期',
  colCount: '字数',
  colShare: '占比',
  showAll: '展开全部',
  showLess: '收起',
  totalRows: '共 {n} 条',

  // 状态
  loading: '正在加载数据…',
  processing: '正在处理日志…',
  noData: '暂无数据',
  noDataHint: '先用 Rime 输入法打字，日志累积后点击「重新处理」',
  errorTitle: '出错了',
  invalidRange: '开始日期不能晚于结束日期',
  invalidDate: '日期格式应为 YYYY-MM-DD',
  reprocessDone: '已处理 {lines} 行日志，更新 {dates} 天记录',
  reprocessSkipped: '{errors} 行无法解析，已跳过',
  reprocessEmpty: '没有新的日志记录',
  demoBadge: '演示数据',
  demoHint: '浏览器预览模式：数据由本地生成，未连接 Rust 后端',

  // 语言
  language: '语言',
} as const;

/** 文案 key */
export type StringKey = keyof typeof zhCN;

/** 某一语言的文案表 */
export type Strings = Record<StringKey, string>;

const zhTW: Strings = {
  appName: 'Rime 字數統計',
  appTagline: '輸入法打字量追蹤',

  groupBy: '分組',
  groupDay: '日',
  groupMonth: '月',
  groupYear: '年',

  dateRange: '時間範圍',
  startDate: '開始日期',
  endDate: '結束日期',
  quickRange: '快捷選擇',
  last7Days: '最近 7 天',
  last30Days: '最近 30 天',
  lastYear: '最近一年',
  allTime: '全部',

  dataSource: '資料來源',
  logFile: '日誌',
  database: '資料庫',

  reprocess: '重新處理',
  reprocessing: '處理中…',
  reprocessHint: '讀取日誌並匯入資料庫',
  retry: '重試',

  totalAll: '總字數',
  totalRange: '範圍內字數',
  dailyAverage: '日均',
  coveredDays: '覆蓋天數',
  peak: '最高',
  trough: '最低',
  buckets: '資料點',
  unitWords: '字',
  unitDays: '天',
  perDay: '/ 天',

  trendTitle: '字數趨勢',
  recordsTitle: '明細記錄',
  chartEmpty: '目前範圍內沒有資料',
  chartEmptyHint: '換個時間範圍，或先用 Rime 輸入法打幾個字',

  colDate: '日期',
  colCount: '字數',
  colShare: '佔比',
  showAll: '展開全部',
  showLess: '收起',
  totalRows: '共 {n} 條',

  loading: '正在載入資料…',
  processing: '正在處理日誌…',
  noData: '暫無資料',
  noDataHint: '先用 Rime 輸入法打字，日誌累積後點擊「重新處理」',
  errorTitle: '發生錯誤',
  invalidRange: '開始日期不能晚於結束日期',
  invalidDate: '日期格式應為 YYYY-MM-DD',
  reprocessDone: '已處理 {lines} 行日誌，更新 {dates} 天記錄',
  reprocessSkipped: '{errors} 行無法解析，已跳過',
  reprocessEmpty: '沒有新的日誌記錄',
  demoBadge: '演示資料',
  demoHint: '瀏覽器預覽模式：資料由本地產生，未連接 Rust 後端',

  language: '語言',
};

const en: Strings = {
  appName: 'Rime Word Counter',
  appTagline: 'Typing volume tracking',

  groupBy: 'Group',
  groupDay: 'Day',
  groupMonth: 'Month',
  groupYear: 'Year',

  dateRange: 'Date range',
  startDate: 'Start date',
  endDate: 'End date',
  quickRange: 'Quick select',
  last7Days: 'Last 7 days',
  last30Days: 'Last 30 days',
  lastYear: 'Last year',
  allTime: 'All time',

  dataSource: 'Data source',
  logFile: 'Log',
  database: 'Database',

  reprocess: 'Reprocess',
  reprocessing: 'Processing…',
  reprocessHint: 'Read the log and merge it into the database',
  retry: 'Retry',

  totalAll: 'Total words',
  totalRange: 'In range',
  dailyAverage: 'Daily average',
  coveredDays: 'Days tracked',
  peak: 'Peak',
  trough: 'Lowest',
  buckets: 'Data points',
  unitWords: '',
  unitDays: 'days',
  perDay: '/ day',

  trendTitle: 'Word count trend',
  recordsTitle: 'Records',
  chartEmpty: 'No data in this range',
  chartEmptyHint: 'Try another range, or type something with Rime first',

  colDate: 'Date',
  colCount: 'Words',
  colShare: 'Share',
  showAll: 'Show all',
  showLess: 'Show less',
  totalRows: '{n} rows',

  loading: 'Loading data…',
  processing: 'Processing log…',
  noData: 'No data yet',
  noDataHint: 'Type with Rime first, then hit “Reprocess” once the log fills up',
  errorTitle: 'Something went wrong',
  invalidRange: 'Start date must not be after end date',
  invalidDate: 'Date format must be YYYY-MM-DD',
  reprocessDone: 'Processed {lines} lines, updated {dates} days',
  reprocessSkipped: '{errors} lines could not be parsed and were skipped',
  reprocessEmpty: 'No new log entries',
  demoBadge: 'Demo data',
  demoHint: 'Browser preview mode: data is generated locally, the Rust backend is not connected',

  language: 'Language',
};

/** 全部语言的文案表 */
export const dictionaries: Record<Lang, Strings> = {
  'zh-CN': zhCN,
  'zh-TW': zhTW,
  en,
};

/** 语言切换器里展示的名称（用各语言自己的写法） */
export const languageOptions: Array<{ value: Lang; label: string }> = [
  { value: 'zh-CN', label: '简体中文' },
  { value: 'zh-TW', label: '繁體中文' },
  { value: 'en', label: 'English' },
];

/** 把 `zh-CN` / `zh_TW` / `zh-Hant` 之类的值收敛到受支持的语言 */
export function normalizeLang(value: string | undefined | null): Lang | null {
  if (!value) return null;
  const normalized = value.toLowerCase().replace(/_/g, '-');

  if (normalized.startsWith('zh')) {
    if (
      normalized.includes('hant') ||
      normalized.includes('tw') ||
      normalized.includes('hk') ||
      normalized.includes('mo')
    ) {
      return 'zh-TW';
    }
    return 'zh-CN';
  }
  if (normalized.startsWith('en')) return 'en';
  return null;
}

/**
 * 取文案并插值：`t(strings, 'reprocessDone', { lines: 12, dates: 3 })`
 * 占位符写法为 `{name}`。
 */
export function format(
  strings: Strings,
  key: StringKey,
  vars?: Record<string, string | number>,
): string {
  const template = strings[key];
  if (!vars) return template;

  return template.replace(/\{(\w+)\}/g, (match, name: string) =>
    name in vars ? String(vars[name]) : match,
  );
}
