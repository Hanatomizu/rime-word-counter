/** 概览指标卡片：总字数 / 范围内 / 日均 / 覆盖天数 */

import { formatDecimal, formatNumber } from '../lib/format';
import type { Strings } from '../i18n/strings';
import type { Summary } from '../types';

interface StatCardsProps {
  summary: Summary;
  strings: Strings;
}

interface Stat {
  label: string;
  value: string;
  unit?: string;
  hint?: string;
}

export function StatCards({ summary, strings }: StatCardsProps) {
  const stats: Stat[] = [
    {
      label: strings.totalAll,
      value: formatNumber(summary.totalAll),
      unit: strings.unitWords,
    },
    {
      label: strings.totalRange,
      value: formatNumber(summary.totalRange),
      unit: strings.unitWords,
    },
    {
      label: strings.dailyAverage,
      value: formatDecimal(summary.average),
      unit: strings.unitWords,
      hint: strings.perDay,
    },
    {
      label: strings.coveredDays,
      value: formatNumber(summary.days),
      unit: strings.unitDays,
      hint: `${strings.buckets} ${formatNumber(summary.buckets)}`,
    },
  ];

  return (
    <section className="stats" aria-label={strings.totalAll}>
      {stats.map((stat) => (
        <article className="stat" key={stat.label}>
          <div className="stat__label">{stat.label}</div>
          <div className="stat__value">
            {stat.value}
            {stat.unit ? <span className="stat__unit">{stat.unit}</span> : null}
          </div>
          {stat.hint ? <div className="stat__hint">{stat.hint}</div> : null}
        </article>
      ))}
    </section>
  );
}
