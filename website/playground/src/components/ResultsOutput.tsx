import { useMemo } from 'react';
import type { AlgoResult } from '../types';

interface ResultsOutputProps {
  algo: string;
  result: AlgoResult | null;
  elapsed: number | null;
  vcount: number;
  edgeCount: number;
  t: (key: string) => string;
}

interface StatBadge {
  label: string;
  value: string;
  accent?: boolean;
}

interface TableRow {
  vertex: number;
  value: string;
  numericValue: number;
  barFraction: number;
  colorIndex?: number;
}

interface ParsedResult {
  badges: StatBadge[];
  table: TableRow[];
  columnLabel: string;
  showBars: boolean;
}

function parseResult(
  result: AlgoResult,
  t: (key: string) => string,
): ParsedResult {
  const badges: StatBadge[] = [];
  const table: TableRow[] = [];
  let columnLabel = '';
  let showBars = false;

  if ('scores' in result) {
    const scores = result.scores;
    const max = Math.max(...scores, 1e-9);
    const min = Math.min(...scores);
    const maxIdx = scores.indexOf(max);
    badges.push({ label: t('result.topNode'), value: `v${maxIdx}`, accent: true });
    badges.push({ label: t('result.maxScore'), value: max.toFixed(4), accent: true });
    columnLabel = t('result.col.score');
    showBars = true;
    scores.forEach((s, i) => {
      table.push({
        vertex: i,
        value: s.toFixed(6),
        numericValue: s,
        barFraction: max > min ? (s - min) / (max - min) : 0.5,
      });
    });
  } else if ('membership' in result) {
    const membership = result.membership;
    const uniqueIds = new Set(membership);
    badges.push({ label: t('result.communities'), value: String(result.count ?? uniqueIds.size), accent: true });
    if (result.modularity != null) {
      badges.push({ label: t('result.modularity'), value: result.modularity.toFixed(4) });
    }
    if (result.codelength != null) {
      badges.push({ label: t('result.codelength'), value: result.codelength.toFixed(4) });
    }
    if (result.nb_clusters != null) {
      badges.push({ label: t('result.clusters'), value: String(result.nb_clusters) });
    }
    columnLabel = t('result.col.community');
    membership.forEach((c, i) => {
      table.push({
        vertex: i,
        value: String(c),
        numericValue: c,
        barFraction: 1,
        colorIndex: c % 8,
      });
    });
  } else if ('order' in result) {
    const order = result.order;
    badges.push({ label: t('result.visited'), value: String(order.length), accent: true });
    columnLabel = t('result.col.order');
    order.forEach((v, i) => {
      table.push({
        vertex: v,
        value: String(i),
        numericValue: i,
        barFraction: order.length > 1 ? i / (order.length - 1) : 1,
      });
    });
    showBars = true;
  }

  return { badges, table, columnLabel, showBars };
}

const COMMUNITY_COLORS = [
  'var(--accent)',
  '#f778ba',
  '#7ee787',
  '#d2a8ff',
  '#ffa657',
  '#ff7b72',
  '#79c0ff',
  '#a5d6ff',
];

export function ResultsOutput({ result, elapsed, vcount, edgeCount, t }: ResultsOutputProps) {
  const parsed = useMemo(() => {
    if (!result) return null;
    return parseResult(result, t);
  }, [result, t]);

  if (!result || !parsed) {
    return (
      <div className="results-output results-empty">
        <span className="results-empty-text">{t('clickToRun')}</span>
      </div>
    );
  }

  const { badges, table, columnLabel, showBars } = parsed;

  const allBadges: StatBadge[] = [
    { label: t('nodes'), value: String(vcount) },
    { label: t('edges'), value: String(edgeCount) },
    ...badges,
  ];
  if (elapsed != null) {
    allBadges.push({ label: t('time'), value: `${elapsed.toFixed(1)}ms` });
  }

  return (
    <div className="results-output">
      <div className="results-badges">
        {allBadges.map((b, i) => (
          <div key={i} className={`result-badge${b.accent ? ' result-badge-accent' : ''}`}>
            <span className="result-badge-value">{b.value}</span>
            <span className="result-badge-label">{b.label}</span>
          </div>
        ))}
      </div>

      {table.length > 0 && (
        <div className="results-table-wrap">
          <table className="results-table">
            <thead>
              <tr>
                <th className="results-th-vertex">{t('result.col.vertex')}</th>
                <th>{columnLabel}</th>
                {showBars && <th className="results-th-bar"></th>}
              </tr>
            </thead>
            <tbody>
              {table.map((row) => (
                <tr key={row.vertex}>
                  <td className="results-td-vertex">v{row.vertex}</td>
                  <td className="results-td-value">
                    {row.colorIndex != null && (
                      <span
                        className="results-color-dot"
                        style={{ background: COMMUNITY_COLORS[row.colorIndex] }}
                      />
                    )}
                    {row.value}
                  </td>
                  {showBars && (
                    <td className="results-td-bar">
                      <div className="results-bar-track">
                        <div
                          className="results-bar-fill"
                          style={{ width: `${(row.barFraction * 100).toFixed(1)}%` }}
                        />
                      </div>
                    </td>
                  )}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
