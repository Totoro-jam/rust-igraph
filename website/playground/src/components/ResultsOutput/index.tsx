import { useMemo } from 'react';
import type { AlgoResult } from '../../types';
import styles from './index.module.css';

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

  if ('hub' in result && 'authority' in result) {
    const hub = result.hub;
    const auth = result.authority;
    const maxHub = Math.max(...hub, 1e-9);
    const maxHubIdx = hub.indexOf(maxHub);
    badges.push({ label: `Hub ${t('result.topNode')}`, value: `v${maxHubIdx}`, accent: true });
    badges.push({ label: `Hub ${t('result.maxScore')}`, value: maxHub.toFixed(4), accent: true });
    columnLabel = 'Hub / Authority';
    showBars = true;
    hub.forEach((h, i) => {
      table.push({
        vertex: i,
        value: `${h.toFixed(4)} / ${auth[i]!.toFixed(4)}`,
        numericValue: h,
        barFraction: maxHub > 0 ? h / maxHub : 0,
      });
    });
  } else if ('scores' in result) {
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
    if (result.quality != null) {
      badges.push({ label: t('result.quality'), value: result.quality.toFixed(4) });
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
  } else if ('diameter' in result) {
    badges.push({ label: t('result.vertices'), value: String(result.vcount), accent: true });
    badges.push({ label: t('result.edgeCount'), value: String(result.ecount), accent: true });
    badges.push({ label: t('result.diameter'), value: String(result.diameter) });
    badges.push({ label: t('result.girth'), value: String(result.girth) });
    badges.push({ label: t('result.triangles'), value: String(result.triangles) });
    badges.push({ label: t('result.connected'), value: result.is_connected ? 'Yes' : 'No' });
    badges.push({ label: t('result.bipartite'), value: result.is_bipartite ? 'Yes' : 'No' });
    badges.push({ label: t('result.directedProp'), value: result.is_directed ? 'Yes' : 'No' });
  } else if ('value' in result) {
    badges.push({ label: t('result.flowValue'), value: (result.value as number).toFixed(4), accent: true });
  } else if ('distances' in result) {
    const distances = (result as { distances: number[] }).distances;
    const finite = distances.filter((d) => Number.isFinite(d));
    const maxDist = finite.length > 0 ? Math.max(...finite) : 1;
    badges.push({ label: t('result.reachable'), value: String(finite.length), accent: true });
    badges.push({ label: t('result.maxDist'), value: maxDist.toFixed(1) });
    columnLabel = t('result.col.distance');
    showBars = true;
    distances.forEach((d, i) => {
      table.push({
        vertex: i,
        value: Number.isFinite(d) ? d.toFixed(1) : '∞',
        numericValue: Number.isFinite(d) ? d : maxDist + 1,
        barFraction: Number.isFinite(d) && maxDist > 0 ? d / maxDist : 0,
      });
    });
  } else if ('vertices' in result) {
    const verts = result.vertices as number[];
    badges.push({ label: t('result.count'), value: String(verts.length), accent: true });
    columnLabel = t('result.col.vertex');
    verts.forEach((v, i) => {
      table.push({
        vertex: v,
        value: String(i),
        numericValue: i,
        barFraction: 1,
      });
    });
  } else if ('degrees' in result) {
    const degrees = (result as { degrees: number[] }).degrees;
    const max = Math.max(...degrees, 1);
    badges.push({ label: t('result.maxDegree'), value: String(max), accent: true });
    badges.push({ label: t('result.minDegree'), value: String(Math.min(...degrees)) });
    columnLabel = t('result.col.degree');
    showBars = true;
    degrees.forEach((d, i) => {
      table.push({
        vertex: i,
        value: String(d),
        numericValue: d,
        barFraction: max > 0 ? d / max : 0,
      });
    });
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
      <div className={`${styles.resultsOutput} ${styles.resultsEmpty}`}>
        <span className={styles.resultsEmptyText}>{t('clickToRun')}</span>
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
    <div className={styles.resultsOutput}>
      <div className={styles.badges}>
        {allBadges.map((b, i) => (
          <div key={i} className={`${styles.badge}${b.accent ? ` ${styles.badgeAccent}` : ''}`}>
            <span className={styles.badgeValue}>{b.value}</span>
            <span className={styles.badgeLabel}>{b.label}</span>
          </div>
        ))}
      </div>

      {table.length > 0 && (
        <div className={styles.tableWrap}>
          <table className={styles.table}>
            <thead>
              <tr>
                <th className={styles.thVertex}>{t('result.col.vertex')}</th>
                <th>{columnLabel}</th>
                {showBars && <th className={styles.thBar}></th>}
              </tr>
            </thead>
            <tbody>
              {table.map((row) => (
                <tr key={row.vertex}>
                  <td className={styles.tdVertex}>v{row.vertex}</td>
                  <td className={styles.tdValue}>
                    {row.colorIndex != null && (
                      <span
                        className={styles.colorDot}
                        style={{ background: COMMUNITY_COLORS[row.colorIndex] }}
                      />
                    )}
                    {row.value}
                  </td>
                  {showBars && (
                    <td className={styles.tdBar}>
                      <div className={styles.barTrack}>
                        <div
                          className={styles.barFill}
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
