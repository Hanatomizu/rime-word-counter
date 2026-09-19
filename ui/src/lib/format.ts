/** 数字格式化工具（金额/字数一类的展示统一走这里，保证千分位与中文单位一致） */

import type { Lang } from '../types';

/** 千分位整数：12345 → "12,345" */
export function formatNumber(value: number): string {
  const rounded = Math.round(value);
  const negative = rounded < 0;
  const digits = Math.abs(rounded).toString();
  let out = '';

  for (let i = 0; i < digits.length; i += 1) {
    if (i > 0 && (digits.length - i) % 3 === 0) out += ',';
    out += digits[i];
  }
  return negative ? `-${out}` : out;
}

/** 一位小数（带千分位）：1234.56 → "1,234.6" */
export function formatDecimal(value: number): string {
  const rounded = Math.round(value * 10) / 10;
  const [int, frac] = rounded.toFixed(1).split('.');
  return `${formatNumber(Number(int))}.${frac}`;
}

/** 去掉多余的小数位：12.0 → "12"，1.25 → "1.3" */
function trimOneDecimal(value: number): string {
  const rounded = Math.round(value * 10) / 10;
  return Number.isInteger(rounded) ? String(rounded) : rounded.toFixed(1);
}

/**
 * 坐标轴刻度用的写法。
 *
 * 刻度必须「同一量纲」，否则会出现 `1万 / 7,500 / 5,000` 这种混搭：
 * 只有当刻度本身长到读不动（≥ 百万）时才切到 k/M 或 万/亿，
 * 其余情况统一用千分位整数。
 */
export function formatAxis(value: number, lang: Lang): string {
  const abs = Math.abs(value);
  const sign = value < 0 ? '-' : '';

  if (abs < 1_000_000) return `${sign}${formatNumber(abs)}`;

  if (lang === 'en') {
    if (abs >= 1_000_000_000) return `${sign}${trimOneDecimal(abs / 1_000_000_000)}B`;
    return `${sign}${trimOneDecimal(abs / 1_000_000)}M`;
  }

  if (abs >= 100_000_000) return `${sign}${trimOneDecimal(abs / 100_000_000)}亿`;
  return `${sign}${trimOneDecimal(abs / 10_000)}万`;
}

/** 百分比：0.1234 → "12.3%" */
export function formatPercent(ratio: number): string {
  if (!Number.isFinite(ratio)) return '0%';
  return `${(ratio * 100).toFixed(1)}%`;
}
