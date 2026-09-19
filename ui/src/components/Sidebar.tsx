/** 左侧栏：品牌、时间范围（输入 + 快捷）、数据源、重新处理 */

import { PRESET_IDS } from '../lib/date';
import type { PresetId } from '../lib/date';
import type { StringKey, Strings } from '../i18n/strings';
import { Logo } from './Logo';

const PRESET_LABELS: Record<PresetId, StringKey> = {
  last7: 'last7Days',
  last30: 'last30Days',
  lastYear: 'lastYear',
  all: 'allTime',
};

interface SidebarProps {
  strings: Strings;
  version: string;
  demo: boolean;
  range: { start: string; end: string };
  onRangeChange: (next: { start: string; end: string }) => void;
  /** 输入框内容非法时标红；首帧未填内容时为 false */
  rangeInvalid: boolean;
  activePreset: PresetId | null;
  onPreset: (id: PresetId) => void;
  dataStart: string;
  dataEnd: string;
  logPath: string;
  dbPath: string;
  busy: boolean;
  onReprocess: () => void;
}

export function Sidebar({
  strings,
  version,
  demo,
  range,
  onRangeChange,
  rangeInvalid,
  activePreset,
  onPreset,
  dataStart,
  dataEnd,
  logPath,
  dbPath,
  busy,
  onReprocess,
}: SidebarProps) {
  return (
    <aside className="sidebar">
      <div className="brand">
        <span className="brand__mark">
          <Logo />
        </span>
        <span className="brand__text">
          <span className="brand__name">{strings.appName}</span>
          <span className="brand__tagline">{strings.appTagline}</span>
        </span>
        {demo && <span className="badge">{strings.demoBadge}</span>}
      </div>

      <div className="sidebar__scroll">
        <section className="side-section">
          <div className="side-label">{strings.dateRange}</div>

          <div className="field">
            <label className="field__label" htmlFor="range-start">
              {strings.startDate}
            </label>
            <input
              id="range-start"
              className={`input${rangeInvalid ? ' input--invalid' : ''}`}
              type="text"
              inputMode="numeric"
              spellCheck={false}
              autoComplete="off"
              placeholder="YYYY-MM-DD"
              value={range.start}
              aria-invalid={rangeInvalid}
              onChange={(event) => onRangeChange({ ...range, start: event.target.value })}
            />
          </div>

          <div className="field">
            <label className="field__label" htmlFor="range-end">
              {strings.endDate}
            </label>
            <input
              id="range-end"
              className={`input${rangeInvalid ? ' input--invalid' : ''}`}
              type="text"
              inputMode="numeric"
              spellCheck={false}
              autoComplete="off"
              placeholder="YYYY-MM-DD"
              value={range.end}
              aria-invalid={rangeInvalid}
              onChange={(event) => onRangeChange({ ...range, end: event.target.value })}
            />
          </div>

          {rangeInvalid && <div className="side-error">{strings.invalidRange}</div>}
        </section>

        <section className="side-section">
          <div className="side-label">{strings.quickRange}</div>
          <div className="chip-row">
            {PRESET_IDS.map((id) => (
              <button
                key={id}
                type="button"
                className={`chip${activePreset === id ? ' chip--active' : ''}`}
                aria-pressed={activePreset === id}
                onClick={() => onPreset(id)}
              >
                {strings[PRESET_LABELS[id]]}
              </button>
            ))}
          </div>
        </section>

        <section className="side-section">
          <div className="side-label">{strings.dataSource}</div>
          <div className="source">
            <div className="source__row">
              <span className="source__key">{strings.logFile}</span>
              <span className="source__value" title={logPath}>
                {logPath || '—'}
              </span>
            </div>
            <div className="source__row">
              <span className="source__key">{strings.database}</span>
              <span className="source__value" title={dbPath}>
                {dbPath || '—'}
              </span>
            </div>
          </div>
        </section>
      </div>

      <div className="sidebar__footer">
        <button
          type="button"
          className="btn btn--primary btn--block"
          onClick={onReprocess}
          disabled={busy}
        >
          {busy ? (
            <>
              <span className="spinner" />
              {strings.reprocessing}
            </>
          ) : (
            <>
              <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
                <path
                  d="M10 6a4 4 0 1 1-1.2-2.85M10 1.6V4.4H7.2"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="1.3"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              </svg>
              {strings.reprocess}
            </>
          )}
        </button>

        <div className="side-hint">{demo ? strings.demoHint : strings.reprocessHint}</div>

        <div className="side-meta">
          <span>v{version}</span>
          <span>{dataStart && dataEnd ? `${dataStart} → ${dataEnd}` : ''}</span>
        </div>
      </div>
    </aside>
  );
}
