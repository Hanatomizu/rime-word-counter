/** 明细记录表：按时间倒序，每行给出字数与占比 */

import { useState } from 'react';

import { formatNumber, formatPercent } from '../lib/format';
import type { Strings } from '../i18n/strings';
import type { Point } from '../types';

/** 折叠状态下展示的行数 */
const COLLAPSED_ROWS = 12;

interface RecordsTableProps {
  points: Point[];
  strings: Strings;
}

export function RecordsTable({ points, strings }: RecordsTableProps) {
  const [expanded, setExpanded] = useState(false);

  if (points.length === 0) return null;

  const total = points.reduce((sum, point) => sum + point.count, 0);
  const peak = points.reduce((max, point) => Math.max(max, point.count), 0);
  const newestFirst = [...points].reverse();
  const visible = expanded ? newestFirst : newestFirst.slice(0, COLLAPSED_ROWS);
  const hidden = newestFirst.length - visible.length;

  return (
    <>
      <div className="table-wrap">
        <table className="table">
          <thead>
            <tr>
              <th scope="col">{strings.colDate}</th>
              <th scope="col" className="is-numeric">
                {strings.colCount}
              </th>
              <th scope="col" className="is-numeric">
                {strings.colShare}
              </th>
            </tr>
          </thead>
          <tbody>
            {visible.map((point) => (
              <tr key={point.label}>
                <td className="table__date">{point.label}</td>
                <td className="is-numeric">
                  <span className="table__count">{formatNumber(point.count)}</span>
                </td>
                <td className="is-numeric">
                  <div className="table__share">
                    <span className="table__share-track">
                      <span
                        className="table__share-fill"
                        style={{ width: `${peak > 0 ? (point.count / peak) * 100 : 0}%` }}
                      />
                    </span>
                    <span className="table__share-text">
                      {formatPercent(total > 0 ? point.count / total : 0)}
                    </span>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {newestFirst.length > COLLAPSED_ROWS && (
        <div className="card__footer">
          <button
            type="button"
            className="btn btn--ghost"
            onClick={() => setExpanded((previous) => !previous)}
          >
            {expanded
              ? strings.showLess
              : `${strings.showAll} (${formatNumber(hidden)} ${strings.unitWords || ''})`.trim()}
          </button>
        </div>
      )}
    </>
  );
}
