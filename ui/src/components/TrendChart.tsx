/**
 * 趋势图：手写 SVG，柱状 + 折线叠加。
 *
 * 不引第三方图表库：这个图表只需要柱、线、hairline 网格和 hover 提示，
 * 自绘可以完全贴合令牌（网格用 hairline、数据用唯一强调色、坐标文字用 ink-tertiary）。
 */

import { useEffect, useId, useMemo, useRef, useState } from 'react';
import type { MouseEvent as ReactMouseEvent } from 'react';

import { shortLabel } from '../lib/date';
import { formatAxis, formatNumber } from '../lib/format';
import type { Strings } from '../i18n/strings';
import type { GroupBy, Lang, Point } from '../types';

const HEIGHT = 288;
const PAD = { top: 18, right: 14, bottom: 28, left: 56 };
/** 网格分成的段数（4 段 = 5 条横线） */
const GRID_SEGMENTS = 4;

interface TrendChartProps {
  points: Point[];
  groupBy: GroupBy;
  lang: Lang;
  strings: Strings;
}

/** 把最大值向上取整到「好看」的刻度：1234 → 2000 */
function niceMax(value: number): number {
  if (value <= 0) return 10;
  const magnitude = 10 ** Math.floor(Math.log10(value));
  const normalized = value / magnitude;

  let step: number;
  if (normalized <= 1) step = 1;
  else if (normalized <= 2) step = 2;
  else if (normalized <= 2.5) step = 2.5;
  else if (normalized <= 5) step = 5;
  else step = 10;

  return step * magnitude;
}

/** 顶部圆角的柱子路径 */
function barPath(x: number, y: number, width: number, height: number, radius: number): string {
  const r = Math.max(0, Math.min(radius, width / 2, height));
  return [
    `M${x},${y + height}`,
    `L${x},${y + r}`,
    `Q${x},${y} ${x + r},${y}`,
    `L${x + width - r},${y}`,
    `Q${x + width},${y} ${x + width},${y + r}`,
    `L${x + width},${y + height}`,
    'Z',
  ].join(' ');
}

export function TrendChart({ points, groupBy, lang, strings }: TrendChartProps) {
  const wrapRef = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(760);
  const [hovered, setHovered] = useState<number | null>(null);
  // React 的 useId 会生成带冒号的 id（如 ":r1:"），去掉冒号再用于 url(#id)
  const gradientId = `chart-fill-${useId().replace(/:/g, '')}`;

  // 跟随容器宽度重绘，保证坐标文字不被拉伸
  useEffect(() => {
    const element = wrapRef.current;
    if (!element) return;

    const observer = new ResizeObserver((entries) => {
      const next = entries[0]?.contentRect.width;
      if (next && next > 0) setWidth(next);
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  const layout = useMemo(() => {
    const innerWidth = Math.max(width - PAD.left - PAD.right, 40);
    const innerHeight = HEIGHT - PAD.top - PAD.bottom;
    const maxCount = points.reduce((max, point) => Math.max(max, point.count), 0);
    const yMax = niceMax(maxCount * 1.12);
    const band = points.length > 0 ? innerWidth / points.length : innerWidth;
    const barWidth = Math.min(band * 0.62, 26);

    const xAt = (index: number) => PAD.left + band * index + band / 2;
    const yAt = (count: number) => PAD.top + innerHeight * (1 - count / yMax);

    const ticks: number[] = [];
    const maxTicks = Math.max(2, Math.floor(innerWidth / 84));
    const step = Math.max(1, Math.ceil(points.length / maxTicks));
    for (let index = 0; index < points.length; index += step) ticks.push(index);

    const lastIndex = points.length - 1;
    if (lastIndex >= 0 && ticks[ticks.length - 1] !== lastIndex) {
      // 末位标签和上一个挨太近就挤掉上一个，避免重叠
      if (ticks.length > 1 && lastIndex - ticks[ticks.length - 1] < step * 0.6) ticks.pop();
      ticks.push(lastIndex);
    }

    return { innerWidth, innerHeight, yMax, band, barWidth, xAt, yAt, ticks };
  }, [points, width]);

  const { innerWidth, innerHeight, yMax, band, barWidth, xAt, yAt, ticks } = layout;
  const baseline = PAD.top + innerHeight;

  const handleMove = (event: ReactMouseEvent<SVGSVGElement>) => {
    if (points.length === 0) return;
    const rect = event.currentTarget.getBoundingClientRect();
    const offsetX = event.clientX - rect.left;
    const index = Math.floor((offsetX - PAD.left) / band);

    setHovered(index >= 0 && index < points.length ? index : null);
  };

  if (points.length === 0) {
    return (
      <div className="chart">
        <div className="chart__empty">
          <div className="chart__empty-title">{strings.chartEmpty}</div>
          <div className="chart__empty-hint">{strings.chartEmptyHint}</div>
        </div>
      </div>
    );
  }

  const hoveredPoint = hovered !== null ? points[hovered] : null;
  const activeIndex = hovered !== null && hovered < points.length ? hovered : null;

  return (
    <div className="chart" ref={wrapRef}>
      <svg
        className="chart__svg"
        width={width}
        height={HEIGHT}
        role="img"
        aria-label={strings.trendTitle}
        onMouseMove={handleMove}
        onMouseLeave={() => setHovered(null)}
      >
        <defs>
          <linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="var(--primary)" stopOpacity="0.22" />
            <stop offset="100%" stopColor="var(--primary)" stopOpacity="0" />
          </linearGradient>
        </defs>

        {/* 横向网格 + Y 轴刻度 */}
        {Array.from({ length: GRID_SEGMENTS + 1 }, (_, line) => {
          const ratio = line / GRID_SEGMENTS;
          const y = PAD.top + innerHeight * ratio;
          const value = yMax * (1 - ratio);
          return (
            <g key={`grid-${line}`}>
              <line className="chart__grid" x1={PAD.left} x2={PAD.left + innerWidth} y1={y} y2={y} />
              <text className="chart__axis-text" x={PAD.left - 10} y={y + 3} textAnchor="end">
                {formatAxis(value, lang)}
              </text>
            </g>
          );
        })}

        {/* 折线下方渐变填充 */}
        {points.length > 1 && (
          <path
            d={[
              `M${xAt(0)},${baseline}`,
              ...points.map((point, index) => `L${xAt(index)},${yAt(point.count)}`),
              `L${xAt(points.length - 1)},${baseline}`,
              'Z',
            ].join(' ')}
            fill={`url(#${gradientId})`}
          />
        )}

        {/* 柱子：太窄时只画折线，避免糊成一片 */}
        {band >= 3 &&
          points.map((point, index) => {
            const height = baseline - yAt(point.count);
            if (height < 0.5) return null;
            return (
              <path
                key={`bar-${point.label}`}
                className={`chart__bar${activeIndex === index ? ' chart__bar--active' : ''}`}
                d={barPath(xAt(index) - barWidth / 2, yAt(point.count), barWidth, height, 3)}
              />
            );
          })}

        {/* 折线 */}
        {points.length > 1 && (
          <polyline
            className="chart__line"
            points={points.map((point, index) => `${xAt(index)},${yAt(point.count)}`).join(' ')}
          />
        )}

        {/* 悬停游标 + 数据点 */}
        {activeIndex !== null && (
          <>
            <line
              className="chart__cursor"
              x1={xAt(activeIndex)}
              x2={xAt(activeIndex)}
              y1={PAD.top}
              y2={baseline}
            />
            <circle className="chart__dot" cx={xAt(activeIndex)} cy={yAt(points[activeIndex].count)} r="3" />
          </>
        )}

        {/* X 轴刻度 */}
        {ticks.map((index) => (
          <text
            key={`tick-${points[index].label}`}
            className="chart__axis-text"
            x={xAt(index)}
            y={baseline + 17}
            textAnchor="middle"
          >
            {shortLabel(points[index].label, groupBy)}
          </text>
        ))}
      </svg>

      {hoveredPoint && activeIndex !== null && (
        <div
          className="chart__tooltip"
          style={{
            left: `${Math.min(Math.max(xAt(activeIndex), 72), Math.max(width - 72, 72))}px`,
            top: `${Math.max(yAt(hoveredPoint.count) - 12, 8)}px`,
          }}
        >
          <div className="chart__tooltip-label">{hoveredPoint.label}</div>
          <div className="chart__tooltip-value">
            {formatNumber(hoveredPoint.count)}
            {strings.unitWords ? ` ${strings.unitWords}` : ''}
          </div>
        </div>
      )}
    </div>
  );
}
