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
  algo: string,
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
  } else if ('cores' in result) {
    const cores = (result as { cores: number[] }).cores;
    const max = Math.max(...cores, 1);
    badges.push({ label: t('result.maxScore'), value: String(max), accent: true });
    columnLabel = t('result.col.core');
    showBars = true;
    cores.forEach((c, i) => {
      table.push({
        vertex: i,
        value: String(c),
        numericValue: c,
        barFraction: max > 0 ? c / max : 0,
      });
    });
  } else if ('values' in result && !('hub' in result)) {
    const values = (result as { values: number[] }).values;
    const max = Math.max(...values, 1);
    const min = Math.min(...values);
    badges.push({ label: t('result.maxScore'), value: String(max), accent: true });
    columnLabel = t('result.col.eccentricity');
    showBars = true;
    values.forEach((v, i) => {
      table.push({
        vertex: i,
        value: String(v),
        numericValue: v,
        barFraction: max > min ? (v - min) / (max - min) : 0.5,
      });
    });
  } else if ('path' in result) {
    const path = (result as { path: number[] }).path;
    badges.push({ label: t('result.pathLength'), value: String(path.length > 0 ? path.length - 1 : 0), accent: true });
    columnLabel = t('result.col.order');
    path.forEach((v, i) => {
      table.push({
        vertex: v,
        value: String(i),
        numericValue: i,
        barFraction: path.length > 1 ? i / (path.length - 1) : 1,
      });
    });
    showBars = true;
  } else if (algo === 'diameter' && 'diameter' in result) {
    const d = (result as { diameter: number | null }).diameter;
    badges.push({ label: t('result.diameterValue'), value: d != null ? String(d) : 'N/A', accent: true });
  } else if ('diameter' in result) {
    const stats = result as import('../../types').AlgoResultStats;
    badges.push({ label: t('result.vertices'), value: String(stats.vcount), accent: true });
    badges.push({ label: t('result.edgeCount'), value: String(stats.ecount), accent: true });
    badges.push({ label: t('result.diameter'), value: String(stats.diameter) });
    badges.push({ label: t('result.girth'), value: String(stats.girth) });
    badges.push({ label: t('result.triangles'), value: String(stats.triangles) });
    badges.push({ label: t('result.connected'), value: stats.is_connected ? 'Yes' : 'No' });
    badges.push({ label: t('result.bipartite'), value: stats.is_bipartite ? 'Yes' : 'No' });
    badges.push({ label: t('result.directedProp'), value: stats.is_directed ? 'Yes' : 'No' });
    if (stats.density != null) {
      badges.push({ label: t('result.density'), value: stats.density.toFixed(4) });
    }
    if (stats.radius != null) {
      badges.push({ label: t('result.radius'), value: String(stats.radius) });
    }
    if (stats.mean_distance != null) {
      badges.push({ label: t('result.meanDistance'), value: stats.mean_distance.toFixed(4) });
    }
    if (stats.mean_degree != null) {
      badges.push({ label: t('result.meanDegree'), value: stats.mean_degree.toFixed(4) });
    }
    if (stats.assortativity != null) {
      badges.push({ label: t('result.assortativity'), value: stats.assortativity.toFixed(4) });
    }
    if (stats.reciprocity != null) {
      badges.push({ label: t('result.reciprocity'), value: stats.reciprocity.toFixed(4) });
    }
  } else if ('colors' in result) {
    const colorResult = result as { colors: number[]; chromatic: number };
    badges.push({ label: t('result.chromatic'), value: String(colorResult.chromatic), accent: true });
    columnLabel = t('result.col.color');
    colorResult.colors.forEach((c, i) => {
      table.push({
        vertex: i,
        value: String(c),
        numericValue: c,
        barFraction: 1,
        colorIndex: c % 8,
      });
    });
  } else if ('value' in result) {
    const label = algo === 'transitivity' ? t('result.transitivity') : t('result.flowValue');
    badges.push({ label, value: (result.value as number).toFixed(4), accent: true });
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
  } else if ('counts' in result) {
    const counts = (result as { counts: number[] }).counts;
    const triadLabels = [
      '003', '012', '102', '021D', '021U', '021C', '111D', '111U',
      '030T', '030C', '201', '120D', '120U', '120C', '210', '300',
    ];
    const max = Math.max(...counts, 1);
    const total = counts.reduce((s, c) => s + c, 0);
    badges.push({ label: 'Total', value: String(total), accent: true });
    columnLabel = t('result.col.triadType');
    showBars = true;
    counts.forEach((c, i) => {
      if (c > 0) {
        table.push({
          vertex: i,
          value: `${triadLabels[i] ?? i}: ${c}`,
          numericValue: c,
          barFraction: max > 0 ? c / max : 0,
        });
      }
    });
  } else if ('permutation' in result) {
    const perm = (result as { permutation: number[] }).permutation;
    badges.push({ label: t('result.vertices'), value: String(perm.length), accent: true });
    columnLabel = t('result.col.permutation');
    perm.forEach((p, i) => {
      table.push({
        vertex: i,
        value: `v${p}`,
        numericValue: p,
        barFraction: perm.length > 1 ? p / (perm.length - 1) : 1,
      });
    });
  } else if ('isomorphic' in result) {
    const isoResult = result as { isomorphic: boolean; mapping: number[] };
    badges.push({ label: t('result.isomorphic'), value: isoResult.isomorphic ? 'Yes' : 'No', accent: true });
    if (isoResult.mapping.length > 0) {
      columnLabel = t('result.col.permutation');
      isoResult.mapping.forEach((m, i) => {
        table.push({
          vertex: i,
          value: `v${m}`,
          numericValue: m,
          barFraction: 1,
        });
      });
    }
  } else if ('edges' in result && 'count' in result) {
    const bridgeResult = result as { edges: [number, number][]; count: number };
    badges.push({ label: t('result.bridges'), value: String(bridgeResult.count), accent: true });
    columnLabel = t('result.col.edge');
    bridgeResult.edges.forEach((e, i) => {
      table.push({
        vertex: i,
        value: `v${e[0]} – v${e[1]}`,
        numericValue: i,
        barFraction: 1,
      });
    });
  } else if ('count' in result && !('membership' in result)) {
    badges.push({ label: t('result.automorphisms'), value: String((result as { count: number }).count), accent: true });
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

export function ResultsOutput({ algo, result, elapsed, vcount, edgeCount, t }: ResultsOutputProps) {
  const parsed = useMemo(() => {
    if (!result) return null;
    return parseResult(algo, result, t);
  }, [algo, result, t]);

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
