/**
 * 应用外壳：状态都集中在这里。
 *
 * 数据流：
 *   bootstrap（系统语言 / 路径）→ request（范围 + 分组）→ getDashboard → 渲染
 * 范围输入有 240ms 防抖，并且只有合法日期才会发出请求。
 */

import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { RecordsTable } from './components/RecordsTable';
import { Select } from './components/Select';
import { Sidebar } from './components/Sidebar';
import { StatCards } from './components/StatCards';
import { TrendChart } from './components/TrendChart';
import { dictionaries, format, languageOptions, normalizeLang } from './i18n/strings';
import { IS_DEMO, getBootstrap, getDashboard, reprocess } from './lib/api';
import { isValidISODate, presetRange, rangeText, todayISO, PRESET_IDS } from './lib/date';
import type { PresetId } from './lib/date';
import { formatNumber } from './lib/format';
import type { Bootstrap, Dashboard, GroupBy, Lang } from './types';

/** 范围输入防抖时长 */
const RANGE_DEBOUNCE_MS = 240;
/** 提示条自动消失时间 */
const TOAST_TTL_MS = 5200;

interface Toast {
  id: number;
  variant: 'info' | 'error';
  title: string;
  text?: string;
}

interface Request {
  start: string;
  end: string;
  groupBy: GroupBy;
}

function messageOf(cause: unknown): string {
  if (cause instanceof Error) return cause.message;
  if (typeof cause === 'string') return cause;
  return String(cause);
}

export function App() {
  const [bootstrap, setBootstrap] = useState<Bootstrap | null>(null);
  const [lang, setLang] = useState<Lang>('en');
  const [rangeInput, setRangeInput] = useState({ start: '', end: '' });
  const [request, setRequest] = useState<Request | null>(null);
  const [dashboard, setDashboard] = useState<Dashboard | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [toasts, setToasts] = useState<Toast[]>([]);

  /** 首次成功加载后，用后端的真实日期回填输入框；之后不再覆盖用户输入 */
  const hydratedRef = useRef(false);
  const toastIdRef = useRef(0);

  const strings = useMemo(() => dictionaries[lang], [lang]);
  const groupBy = request?.groupBy ?? 'day';

  const pushToast = useCallback((toast: Omit<Toast, 'id'>) => {
    toastIdRef.current += 1;
    const id = toastIdRef.current;
    setToasts((previous) => [...previous, { ...toast, id }]);
    window.setTimeout(() => {
      setToasts((previous) => previous.filter((item) => item.id !== id));
    }, TOAST_TTL_MS);
  }, []);

  // ---- 启动：探测语言 + 首次加载 ----
  useEffect(() => {
    let cancelled = false;

    (async () => {
      try {
        const info = await getBootstrap();
        if (cancelled) return;

        setBootstrap(info);
        setLang(normalizeLang(info.language) ?? normalizeLang(navigator.language) ?? 'en');
        setRequest({ start: '', end: '', groupBy: 'day' });
      } catch (cause) {
        if (!cancelled) {
          setError(messageOf(cause));
          setLoading(false);
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, []);

  // ---- 请求仪表盘 ----
  useEffect(() => {
    if (!request) return;
    let cancelled = false;

    setLoading(true);
    getDashboard({
      start: request.start || undefined,
      end: request.end || undefined,
      groupBy: request.groupBy,
    })
      .then((data) => {
        if (cancelled) return;
        setDashboard(data);
        setError(null);
        if (!hydratedRef.current) {
          hydratedRef.current = true;
          setRangeInput({ start: data.rangeStart, end: data.rangeEnd });
        }
      })
      .catch((cause) => {
        if (!cancelled) setError(messageOf(cause));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [request]);

  // ---- 输入框 → 请求（防抖 + 日期校验）----
  useEffect(() => {
    if (!hydratedRef.current) return;
    if (!isValidISODate(rangeInput.start) || !isValidISODate(rangeInput.end)) return;

    const timer = window.setTimeout(() => {
      setRequest((previous) => {
        if (previous && previous.start === rangeInput.start && previous.end === rangeInput.end) {
          return previous;
        }
        return {
          start: rangeInput.start,
          end: rangeInput.end,
          groupBy: previous?.groupBy ?? 'day',
        };
      });
    }, RANGE_DEBOUNCE_MS);

    return () => window.clearTimeout(timer);
  }, [rangeInput]);

  // ---- 窗口标题跟随语言 ----
  useEffect(() => {
    document.title = strings.appName;
    document.documentElement.lang = lang;

    if (IS_DEMO) return;
    void import('@tauri-apps/api/window')
      .then(({ getCurrentWindow }) => getCurrentWindow().setTitle(strings.appName))
      .catch(() => {
        /* 标题更新失败不影响功能 */
      });
  }, [strings, lang]);

  const handleGroupBy = (next: GroupBy) => {
    setRequest((previous) =>
      previous
        ? { ...previous, groupBy: next }
        : { start: rangeInput.start, end: rangeInput.end, groupBy: next },
    );
  };

  const activePreset = useMemo<PresetId | null>(() => {
    if (!dashboard) return null;
    for (const id of PRESET_IDS) {
      const candidate = presetRange(id, dashboard.dataStart, dashboard.dataEnd);
      if (candidate.start === rangeInput.start && candidate.end === rangeInput.end) return id;
    }
    return null;
  }, [dashboard, rangeInput]);

  const handlePreset = (id: PresetId) => {
    const next = presetRange(id, dashboard?.dataStart ?? todayISO(), dashboard?.dataEnd ?? todayISO());
    setRangeInput(next);
    setRequest((previous) => ({ start: next.start, end: next.end, groupBy: previous?.groupBy ?? 'day' }));
  };

  const handleReprocess = async () => {
    setBusy(true);
    try {
      const outcome = await reprocess({
        start: request?.start || undefined,
        end: request?.end || undefined,
        groupBy: request?.groupBy ?? 'day',
      });

      // 之前没有数据（或选了「全部」）时，把范围扩到新的完整区间，
      // 否则刚导入的历史数据会落在当前筛选之外看不见。
      const expandToAll =
        dashboard === null ||
        dashboard.summary.totalAll === 0 ||
        activePreset === 'all';

      const nextRange = expandToAll
        ? { start: outcome.dashboard.dataStart, end: outcome.dashboard.dataEnd }
        : { start: request?.start ?? '', end: request?.end ?? '' };

      setDashboard(outcome.dashboard);
      setLoading(false);
      setError(null);
      hydratedRef.current = true;
      setRangeInput(nextRange);

      if (nextRange.start !== (request?.start ?? '') || nextRange.end !== (request?.end ?? '')) {
        setRequest((previous) => ({ ...nextRange, groupBy: previous?.groupBy ?? 'day' }));
      }

      const { report } = outcome;
      pushToast({
        variant: 'info',
        title:
          report.linesRead === 0
            ? strings.reprocessEmpty
            : format(strings, 'reprocessDone', {
                lines: report.linesRead,
                dates: report.datesUpdated,
              }),
        text:
          report.parseErrors > 0
            ? format(strings, 'reprocessSkipped', { errors: report.parseErrors })
            : undefined,
      });
    } catch (cause) {
      pushToast({ variant: 'error', title: strings.errorTitle, text: messageOf(cause) });
    } finally {
      setBusy(false);
    }
  };

  const rangeStartOk = isValidISODate(rangeInput.start);
  const rangeEndOk = isValidISODate(rangeInput.end);
  const rangeValid = rangeStartOk && rangeEndOk && rangeInput.start <= rangeInput.end;
  /**
   * 首帧两个输入框还是空的（数据尚未返回），此时不该标红，
   * 否则启动瞬间会闪一下错误态。只有用户真的填了东西才提示非法。
   */
  const rangeInvalid = !rangeValid && rangeInput.start !== '' && rangeInput.end !== '';

  const hasAnyData = (dashboard?.summary.totalAll ?? 0) > 0;

  return (
    <div className="shell">
      <Sidebar
        strings={strings}
        version={bootstrap?.version ?? '—'}
        demo={IS_DEMO}
        range={rangeInput}
        onRangeChange={setRangeInput}
        rangeInvalid={rangeInvalid}
        activePreset={activePreset}
        onPreset={handlePreset}
        dataStart={dashboard?.dataStart ?? ''}
        dataEnd={dashboard?.dataEnd ?? ''}
        logPath={bootstrap?.logPath ?? ''}
        dbPath={bootstrap?.dbPath ?? ''}
        busy={busy}
        onReprocess={() => void handleReprocess()}
      />

      <div className="main">
        <header className="topbar">
          <div className="topbar__left">
            <div className="topbar__title">
              {dashboard ? rangeText(dashboard.rangeStart, dashboard.rangeEnd) : strings.loading}
            </div>
            <div className="topbar__meta">
              {dashboard
                ? `${formatNumber(dashboard.summary.days)} ${strings.unitDays} · ${formatNumber(
                    dashboard.summary.buckets,
                  )} ${strings.buckets}`
                : strings.appTagline}
            </div>
          </div>

          <div className="topbar__right">
            <div className="topbar__total">
              <span className="topbar__total-value">
                {formatNumber(dashboard?.summary.totalAll ?? 0)}
              </span>
              <span className="topbar__total-label">{strings.totalAll}</span>
            </div>
            <Select
              value={lang}
              options={languageOptions}
              onChange={setLang}
              label={strings.language}
            />
          </div>
        </header>

        {loading && <div className="loading-bar" />}


        <div className="content">
          {error && hasAnyData && (
            <div className="banner">
              <span className="banner__text">
                {strings.errorTitle}：{error}
              </span>
              <button type="button" className="btn btn--secondary" onClick={() => setRequest((p) => (p ? { ...p } : p))}>
                {strings.retry}
              </button>
            </div>
          )}

          {!dashboard && loading && (
            <div className="state">
              <span className="spinner" />
              <div className="state__title">{strings.loading}</div>
            </div>
          )}

          {!dashboard && !loading && (
            <div className="state">
              <div className="state__title">{strings.errorTitle}</div>
              <div className="state__text">{error ?? strings.noDataHint}</div>
              {error && <div className="state__detail">{error}</div>}
              <button
                type="button"
                className="btn btn--secondary"
                onClick={() => setRequest({ start: '', end: '', groupBy })}
              >
                {strings.retry}
              </button>
            </div>
          )}

          {dashboard && !hasAnyData && (
            <div className="state">
              <div className="state__title">{strings.noData}</div>
              <div className="state__text">{strings.noDataHint}</div>
            </div>
          )}

          {dashboard && hasAnyData && (
            <>
              <StatCards summary={dashboard.summary} strings={strings} />

              <section className="card">
                <div className="card__header">
                  <div className="card__titles">
                    <div className="card__title">{strings.trendTitle}</div>
                    <div className="card__subtitle">
                      {rangeText(dashboard.rangeStart, dashboard.rangeEnd)}
                    </div>
                  </div>
                  <div className="card__actions">
                    <div className="segmented" role="group" aria-label={strings.groupBy}>
                      {(
                        [
                          ['day', strings.groupDay],
                          ['month', strings.groupMonth],
                          ['year', strings.groupYear],
                        ] as Array<[GroupBy, string]>
                      ).map(([value, label]) => (
                        <button
                          key={value}
                          type="button"
                          className={`segmented__item${
                            groupBy === value ? ' segmented__item--active' : ''
                          }`}
                          aria-pressed={groupBy === value}
                          onClick={() => handleGroupBy(value)}
                        >
                          {label}
                        </button>
                      ))}
                    </div>
                  </div>
                </div>

                <TrendChart
                  points={dashboard.points}
                  groupBy={dashboard.groupBy}
                  lang={lang}
                  strings={strings}
                />

                <div className="chart__footer">
                  <div className="chart__footer-item">
                    <span className="chart__footer-label">{strings.peak}</span>
                    <span className="chart__footer-value">
                      {formatNumber(dashboard.summary.max)}
                    </span>
                  </div>
                  <div className="chart__footer-item">
                    <span className="chart__footer-label">{strings.trough}</span>
                    <span className="chart__footer-value">
                      {formatNumber(dashboard.summary.min)}
                    </span>
                  </div>
                  <div className="chart__footer-item">
                    <span className="chart__footer-label">{strings.dailyAverage}</span>
                    <span className="chart__footer-value">
                      {formatNumber(dashboard.summary.average)}
                    </span>
                  </div>
                </div>
              </section>

              <section className="card">
                <div className="card__header">
                  <div className="card__titles">
                    <div className="card__title">{strings.recordsTitle}</div>
                  </div>
                  <div className="card__subtitle">
                    {format(strings, 'totalRows', { n: dashboard.points.length })}
                  </div>
                </div>
                <RecordsTable points={dashboard.points} strings={strings} />
              </section>
            </>
          )}
        </div>
      </div>

      {toasts.length > 0 && (
        <div className="toast-stack" role="status" aria-live="polite">
          {toasts.map((toast) => (
            <div
              key={toast.id}
              className={`toast${toast.variant === 'error' ? ' toast--error' : ''}`}
            >
              <div className="toast__title">{toast.title}</div>
              {toast.text && <div className="toast__text">{toast.text}</div>}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
