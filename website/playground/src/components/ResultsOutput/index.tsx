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
  } else if (algo === 'avg_nearest_neighbor_degree' && 'scores' in result) {
    const raw = result.scores as (number | null)[];
    const scores = raw.map(s => s ?? 0);
    const max = Math.max(...scores, 1e-9);
    const min = Math.min(...scores);
    badges.push({ label: t('result.topNode'), value: `v${scores.indexOf(max)}`, accent: true });
    badges.push({ label: t('result.maxScore'), value: max.toFixed(4), accent: true });
    columnLabel = t('result.knnDegree');
    showBars = true;
    scores.forEach((s, i) => {
      table.push({
        vertex: i,
        value: raw[i] === null ? 'N/A' : s.toFixed(4),
        numericValue: s,
        barFraction: max > min ? (s - min) / (max - min) : 0.5,
      });
    });
  } else if (algo === 'clustering_coefficients' && 'scores' in result) {
    const raw = result.scores as (number | null)[];
    const scores = raw.map(s => s ?? 0);
    const max = Math.max(...scores, 1e-9);
    const min = Math.min(...scores);
    const avg = scores.length > 0 ? scores.reduce((a, b) => a + b, 0) / scores.length : 0;
    badges.push({ label: t('result.maxScore'), value: max.toFixed(4), accent: true });
    badges.push({ label: 'Avg', value: avg.toFixed(4) });
    columnLabel = t('result.clusteringCoeff');
    showBars = true;
    scores.forEach((s, i) => {
      table.push({
        vertex: i,
        value: raw[i] === null ? 'N/A' : s.toFixed(6),
        numericValue: s,
        barFraction: max > min ? (s - min) / (max - min) : 0.5,
      });
    });
  } else if ('scores' in result) {
    const scores = result.scores as number[];
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
    const r = result as import('../../types').AlgoResultMembership;
    const uniqueIds = new Set(membership);
    badges.push({ label: t('result.communities'), value: String(r.count ?? uniqueIds.size), accent: true });
    if (r.modularity != null) {
      badges.push({ label: t('result.modularity'), value: r.modularity.toFixed(4) });
    }
    if (r.codelength != null) {
      badges.push({ label: t('result.codelength'), value: r.codelength.toFixed(4) });
    }
    if (r.quality != null) {
      badges.push({ label: t('result.quality'), value: r.quality.toFixed(4) });
    }
    if (r.nb_clusters != null) {
      badges.push({ label: t('result.clusters'), value: String(r.nb_clusters) });
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
    const props = (result as unknown as Record<string, unknown>).properties as Record<string, boolean> | undefined;
    if (props) {
      const propLabels: Record<string, string> = {
        is_tree: 'Tree', is_forest: 'Forest', is_dag: 'DAG', is_acyclic: 'Acyclic',
        is_complete: 'Complete', is_biconnected: 'Biconnected', is_tournament: 'Tournament',
        is_cubic: 'Cubic', is_cycle: 'Cycle', is_path: 'Path', is_star: 'Star',
        is_wheel: 'Wheel', is_perfect: 'Perfect', is_triangle_free: 'Triangle-Free',
        is_outerplanar: 'Outerplanar',
      };
      for (const [key, label] of Object.entries(propLabels)) {
        if (props[key]) {
          badges.push({ label, value: 'Yes', accent: false });
        }
      }
    }
  } else if (algo === 'bipartite_check' && 'is_bipartite' in result) {
    const bp = result as { is_bipartite: boolean; types: number[] };
    badges.push({ label: t('result.isBipartite'), value: bp.is_bipartite ? 'Yes' : 'No', accent: true });
    if (bp.is_bipartite && bp.types.length > 0) {
      columnLabel = t('result.col.type');
      bp.types.forEach((ty, i) => {
        table.push({
          vertex: i,
          value: ty === 0 ? 'A' : 'B',
          numericValue: ty,
          barFraction: 1,
          colorIndex: ty % 8,
        });
      });
    }
  } else if (algo === 'maximum_cut' && 'partition' in result) {
    const mc = result as { partition: boolean[]; cut_value: number };
    badges.push({ label: t('result.cutValue'), value: String(mc.cut_value), accent: true });
    columnLabel = t('result.col.partition');
    mc.partition.forEach((side, i) => {
      table.push({
        vertex: i,
        value: side ? 'S' : 'T',
        numericValue: side ? 1 : 0,
        barFraction: 1,
        colorIndex: side ? 0 : 1,
      });
    });
  } else if (algo === 'is_eulerian' && 'has_path' in result) {
    const eu = result as { has_path: boolean; has_cycle: boolean };
    badges.push({ label: t('result.hasEulerianPath'), value: eu.has_path ? 'Yes' : 'No', accent: true });
    badges.push({ label: t('result.hasEulerianCycle'), value: eu.has_cycle ? 'Yes' : 'No', accent: true });
  } else if (algo === 'find_cycle' && 'found' in result) {
    const fc = result as { vertices: number[]; found: boolean };
    badges.push({ label: t('result.cycleFound'), value: fc.found ? 'Yes' : 'No', accent: true });
    if (fc.found) {
      badges.push({ label: t('result.count'), value: String(fc.vertices.length) });
      columnLabel = t('result.col.order');
      fc.vertices.forEach((v, i) => {
        table.push({
          vertex: v,
          value: String(i),
          numericValue: i,
          barFraction: fc.vertices.length > 1 ? i / (fc.vertices.length - 1) : 1,
        });
      });
      showBars = true;
    }
  } else if (algo === 'biconnected_components' && 'components' in result) {
    const bc = result as { count: number; components: number[][] };
    badges.push({ label: t('result.biconnectedCount'), value: String(bc.count), accent: true });
    bc.components.forEach((comp, i) => {
      table.push({
        vertex: i,
        value: comp.map(v => `v${v}`).join(', '),
        numericValue: comp.length,
        barFraction: 1,
        colorIndex: i % 8,
      });
    });
  } else if (algo === 'all_simple_paths' && 'paths' in result) {
    const sp = result as { paths: number[][]; count: number };
    badges.push({ label: t('result.pathCount'), value: String(sp.count), accent: true });
    sp.paths.forEach((p, i) => {
      table.push({
        vertex: i,
        value: p.map(v => `v${v}`).join(' → '),
        numericValue: p.length,
        barFraction: 1,
      });
    });
  } else if (algo === 'k_shortest_paths' && 'paths' in result) {
    const kp = result as { paths: { vertices: number[]; weight: number }[]; count: number };
    badges.push({ label: t('result.kPathCount'), value: String(kp.count), accent: true });
    kp.paths.forEach((p, i) => {
      table.push({
        vertex: i,
        value: `[${p.weight.toFixed(1)}] ${p.vertices.map(v => `v${v}`).join(' → ')}`,
        numericValue: p.weight,
        barFraction: 1,
      });
    });
  } else if (algo === 'cohesive_blocks' && 'blocks' in result) {
    const cb = result as { blocks: number[][]; cohesion: number[]; count: number };
    badges.push({ label: t('result.cohesiveBlockCount'), value: String(cb.count), accent: true });
    cb.blocks.forEach((block, i) => {
      table.push({
        vertex: i,
        value: `[${cb.cohesion[i]}] ${block.map(v => `v${v}`).join(', ')}`,
        numericValue: cb.cohesion[i] ?? 0,
        barFraction: 1,
        colorIndex: i % 8,
      });
    });
  } else if (algo === 'similarity_jaccard' && 'matrix' in result) {
    const sim = result as { matrix: number[][]; size: number };
    badges.push({ label: t('result.vertices'), value: String(sim.size), accent: true });
    columnLabel = t('result.similarity');
    sim.matrix.forEach((row, i) => {
      const maxSim = Math.max(...row.filter((_, j) => j !== i), 0);
      const bestJ = row.findIndex((v, j) => j !== i && v === maxSim);
      table.push({
        vertex: i,
        value: bestJ >= 0 ? `v${bestJ} (${maxSim.toFixed(4)})` : 'N/A',
        numericValue: maxSim,
        barFraction: maxSim,
      });
    });
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
    const scalarLabels: Record<string, string> = {
      transitivity: 'result.transitivity',
      global_efficiency: 'result.efficiency',
      degeneracy: 'result.degeneracyValue',
      mincut_value: 'result.mincutValue',
      vertex_disjoint_paths: 'result.disjointPaths',
      edge_disjoint_paths: 'result.disjointPaths',
      chromatic_number: 'result.chromaticNumber',
      average_path_length: 'result.avgPathLength',
    };
    const label = t(scalarLabels[algo] ?? 'result.flowValue');
    const v = result.value as number;
    badges.push({ label, value: Number.isInteger(v) ? String(v) : v.toFixed(4), accent: true });
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
  } else if (algo === 'feedback_arc_set' && 'edges' in result && 'count' in result) {
    const fasResult = result as { edges: number[]; count: number };
    badges.push({ label: t('result.count'), value: String(fasResult.count), accent: true });
    columnLabel = t('result.col.edge');
    fasResult.edges.forEach((edgeId, i) => {
      table.push({
        vertex: i,
        value: `e${edgeId}`,
        numericValue: edgeId,
        barFraction: 1,
      });
    });
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
  } else if ('cycles' in result) {
    const cycResult = result as { cycles: number[][]; count: number };
    badges.push({ label: t('result.cycleCount'), value: String(cycResult.count), accent: true });
    cycResult.cycles.forEach((c, i) => {
      table.push({
        vertex: i,
        value: c.map(v => `v${v}`).join(' → '),
        numericValue: c.length,
        barFraction: 1,
      });
    });
  } else if ('triangles' in result && Array.isArray((result as { triangles: unknown }).triangles)) {
    const triResult = result as { triangles: [number, number, number][]; count: number };
    badges.push({ label: t('result.triangleCount'), value: String(triResult.count), accent: true });
    triResult.triangles.forEach((tri, i) => {
      table.push({
        vertex: i,
        value: `v${tri[0]} – v${tri[1]} – v${tri[2]}`,
        numericValue: i,
        barFraction: 1,
      });
    });
  } else if ('trussness' in result) {
    const tResult = result as { trussness: number[] };
    const max = Math.max(...tResult.trussness, 1);
    badges.push({ label: 'Max', value: String(max), accent: true });
    columnLabel = t('result.col.trussness');
    showBars = true;
    tResult.trussness.forEach((t_val, i) => {
      table.push({
        vertex: i,
        value: String(t_val),
        numericValue: t_val,
        barFraction: max > 0 ? t_val / max : 0,
      });
    });
  } else if ('generators' in result) {
    const genResult = result as { generators: number[][]; count: number };
    badges.push({ label: t('result.generators'), value: String(genResult.count), accent: true });
    genResult.generators.forEach((gen, i) => {
      table.push({
        vertex: i,
        value: gen.map((v, j) => j !== v ? `${j}→${v}` : '').filter(Boolean).join(', '),
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
