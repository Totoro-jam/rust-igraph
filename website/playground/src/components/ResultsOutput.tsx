import { useMemo } from 'react';
import type { AlgoId, AlgoResult, AlgoResultScores, AlgoResultMembership, AlgoResultOrder } from '../types';

interface ResultsOutputProps {
  algo: AlgoId;
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
}

function parseAlgoResult(
  algo: AlgoId,
  result: AlgoResult,
  t: (key: string) => string,
): ParsedResult {
  const badges: StatBadge[] = [];
  const table: TableRow[] = [];
  let columnLabel = '';

  if (algo === 'pagerank' && 'scores' in result) {
    const r = result as AlgoResultScores;
    const max = Math.max(...r.scores);
    const maxIdx = r.scores.indexOf(max);
    badges.push({ label: t('result.topNode'), value: `v${maxIdx}`, accent: true });
    badges.push({ label: t('result.maxScore'), value: max.toFixed(4), accent: true });
    columnLabel = t('result.col.score');
    r.scores.forEach((s, i) => {
      table.push({
        vertex: i,
        value: s.toFixed(6),
        numericValue: s,
        barFraction: max > 0 ? s / max : 0,
      });
    });
  } else if (algo === 'betweenness' && 'scores' in result) {
    const r = result as AlgoResultScores;
    const max = Math.max(...r.scores);
    const maxIdx = r.scores.indexOf(max);
    badges.push({ label: t('result.topNode'), value: `v${maxIdx}`, accent: true });
    badges.push({ label: t('result.maxScore'), value: max.toFixed(4), accent: true });
    columnLabel = t('result.col.centrality');
    r.scores.forEach((s, i) => {
      table.push({
        vertex: i,
        value: s.toFixed(4),
        numericValue: s,
        barFraction: max > 0 ? s / max : 0,
      });
    });
  } else if ((algo === 'louvain' || algo === 'infomap' || algo === 'spinglass') && 'membership' in result) {
    const r = result as AlgoResultMembership;
    const uniqueComms = new Set(r.membership);
    badges.push({ label: t('result.communities'), value: String(uniqueComms.size), accent: true });
    if (r.modularity != null) {
      badges.push({ label: t('result.modularity'), value: r.modularity.toFixed(4) });
    }
    if (r.codelength != null) {
      badges.push({ label: t('result.codelength'), value: r.codelength.toFixed(4) });
    }
    if (r.nb_clusters != null) {
      badges.push({ label: t('result.clusters'), value: String(r.nb_clusters) });
    }
    columnLabel = t('result.col.community');
    const maxComm = Math.max(...r.membership, 0);
    r.membership.forEach((c, i) => {
      table.push({
        vertex: i,
        value: String(c),
        numericValue: c,
        barFraction: 1,
        colorIndex: c % 8,
      });
    });
    void maxComm;
  } else if (algo === 'components' && 'membership' in result) {
    const r = result as AlgoResultMembership;
    const uniqueComps = new Set(r.membership);
    badges.push({ label: t('result.componentCount'), value: String(r.count ?? uniqueComps.size), accent: true });
    columnLabel = t('result.col.component');
    r.membership.forEach((c, i) => {
      table.push({
        vertex: i,
        value: String(c),
        numericValue: c,
        barFraction: 1,
        colorIndex: c % 8,
      });
    });
  } else if (algo === 'bfs' && 'order' in result) {
    const r = result as AlgoResultOrder;
    badges.push({ label: t('result.visited'), value: String(r.order.length), accent: true });
    columnLabel = t('result.col.order');
    r.order.forEach((v, i) => {
      table.push({
        vertex: v,
        value: String(i),
        numericValue: i,
        barFraction: r.order.length > 1 ? i / (r.order.length - 1) : 1,
      });
    });
  }

  return { badges, table, columnLabel };
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

export function ResultsOutput({ algo, result, elapsed, vcount, edgeCount, t }: ResultsOutputProps) {
  const parsed = useMemo(() => {
    if (!result) return null;
    return parseAlgoResult(algo, result, t);
  }, [algo, result, t]);

  if (!result || !parsed) {
    return (
      <div className="results-output results-empty">
        <span className="results-empty-text">{t('clickToRun')}</span>
      </div>
    );
  }

  const { badges, table, columnLabel } = parsed;

  const allBadges: StatBadge[] = [
    { label: t('nodes'), value: String(vcount) },
    { label: t('edges'), value: String(edgeCount) },
    ...badges,
  ];
  if (elapsed != null) {
    allBadges.push({ label: t('time'), value: `${elapsed.toFixed(1)}ms` });
  }

  const isCommunityAlgo = algo === 'louvain' || algo === 'infomap' || algo === 'spinglass' || algo === 'components';

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
                {!isCommunityAlgo && <th className="results-th-bar"></th>}
              </tr>
            </thead>
            <tbody>
              {table.map((row) => (
                <tr key={row.vertex}>
                  <td className="results-td-vertex">v{row.vertex}</td>
                  <td className="results-td-value">
                    {isCommunityAlgo && row.colorIndex != null && (
                      <span
                        className="results-color-dot"
                        style={{ background: COMMUNITY_COLORS[row.colorIndex] }}
                      />
                    )}
                    {row.value}
                  </td>
                  {!isCommunityAlgo && (
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
