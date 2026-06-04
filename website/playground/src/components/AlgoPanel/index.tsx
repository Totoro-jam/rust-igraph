import { useState, useCallback } from 'react';
import type { AlgoId, AlgoParams, LayoutId } from '../../types';
import styles from './index.module.css';

interface AlgoCategory {
  key: string;
  icon: string;
  algos: AlgoId[];
}

const ALGO_CATEGORIES: AlgoCategory[] = [
  {
    key: 'centrality',
    icon: '◎',
    algos: ['pagerank', 'betweenness', 'closeness', 'eigenvector', 'harmonic', 'hits', 'katz', 'edge_betweenness_centrality'],
  },
  {
    key: 'community',
    icon: '⬡',
    algos: [
      'louvain', 'leiden', 'infomap', 'label_propagation', 'walktrap',
      'fast_greedy', 'leading_eigenvector', 'edge_betweenness', 'spinglass', 'fluid',
    ],
  },
  {
    key: 'traversal',
    icon: '⇢',
    algos: ['bfs', 'dfs', 'dijkstra', 'shortest_path', 'random_walk', 'max_flow', 'topological_sort'],
  },
  {
    key: 'structure',
    icon: '⊞',
    algos: ['components', 'scc', 'articulation_points', 'bridges', 'coloring', 'triad_census'],
  },
  {
    key: 'metrics',
    icon: '▦',
    algos: ['graph_stats', 'degree_sequence', 'coreness', 'eccentricity', 'constraint', 'diameter', 'transitivity'],
  },
  {
    key: 'isomorphism',
    icon: '≅',
    algos: ['canonical_permutation', 'count_automorphisms', 'isomorphism'],
  },
];

const STORAGE_KEY = 'playground-algo-collapsed';

function loadCollapsed(): Record<string, boolean> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? JSON.parse(raw) : {};
  } catch {
    return {};
  }
}

const LAYOUT_OPTIONS: LayoutId[] = ['fr', 'kamada_kawai', 'circle', 'random', 'grid', 'star'];

interface AlgoPanelProps {
  algo: AlgoId;
  params: AlgoParams;
  layoutId: LayoutId;
  running: boolean;
  onAlgoChange: (algo: AlgoId) => void;
  onParamsChange: (params: AlgoParams) => void;
  onLayoutChange: (layout: LayoutId) => void;
  onRun: () => void;
  t: (key: string) => string;
}

export function AlgoPanel({
  algo,
  params,
  layoutId,
  running,
  onAlgoChange,
  onParamsChange,
  onLayoutChange,
  onRun,
  t,
}: AlgoPanelProps) {
  const [collapsed, setCollapsed] = useState<Record<string, boolean>>(loadCollapsed);

  const toggleGroup = useCallback((key: string) => {
    setCollapsed((prev) => {
      const next = { ...prev, [key]: !prev[key] };
      try { localStorage.setItem(STORAGE_KEY, JSON.stringify(next)); } catch { /* noop */ }
      return next;
    });
  }, []);

  return (
    <>
      <div className={styles.algoList}>
        {ALGO_CATEGORIES.map((cat) => {
          const isCollapsed = collapsed[cat.key] ?? false;
          const hasActive = cat.algos.includes(algo);
          return (
            <div key={cat.key} className={styles.algoGroup}>
              <button
                className={`${styles.groupHeader}${hasActive && isCollapsed ? ` ${styles.groupHeaderActive}` : ''}`}
                onClick={() => toggleGroup(cat.key)}
              >
                <span className={styles.groupIcon}>{cat.icon}</span>
                <span className={styles.groupLabel}>
                  {t(`cat.${cat.key}`)}
                  <span className={styles.groupCount}>{cat.algos.length}</span>
                </span>
                <svg
                  className={`${styles.groupChevron}${isCollapsed ? ` ${styles.groupChevronCollapsed}` : ''}`}
                  width="12"
                  height="12"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <polyline points="6 9 12 15 18 9" />
                </svg>
              </button>
              <div className={`${styles.groupBody}${isCollapsed ? ` ${styles.groupBodyCollapsed}` : ''}`}>
                <div className={styles.groupBodyInner}>
                  {cat.algos.map((id) => {
                    const label = t(`algo.${id}`);
                    return (
                      <label key={id} className={`${styles.algoOption}${algo === id ? ` ${styles.active}` : ''}`} title={label}>
                        <input
                          type="radio"
                          name="algo"
                          value={id}
                          checked={algo === id}
                          onChange={() => onAlgoChange(id)}
                        />
                        <span>{label}</span>
                      </label>
                    );
                  })}
                </div>
              </div>
            </div>
          );
        })}
      </div>

      <div className={styles.algoParams}>
        {algo === 'pagerank' && (
          <div className={styles.paramRow}>
            <label>{t('param.damping')}</label>
            <input
              type="number"
              min="0"
              max="1"
              step="0.05"
              value={params.damping ?? 0.85}
              onChange={(e) =>
                onParamsChange({ ...params, damping: parseFloat(e.target.value) })
              }
            />
          </div>
        )}
        {(algo === 'bfs' || algo === 'dfs' || algo === 'dijkstra' || algo === 'max_flow' || algo === 'shortest_path' || algo === 'random_walk') && (
          <div className={styles.paramRow}>
            <label>{t('param.source')}</label>
            <input
              type="number"
              min="0"
              step="1"
              value={params.source ?? 0}
              onChange={(e) =>
                onParamsChange({ ...params, source: parseInt(e.target.value, 10) })
              }
            />
          </div>
        )}
        {(algo === 'max_flow' || algo === 'shortest_path') && (
          <div className={styles.paramRow}>
            <label>{t('param.target')}</label>
            <input
              type="number"
              min="0"
              step="1"
              value={params.target ?? 1}
              onChange={(e) =>
                onParamsChange({ ...params, target: parseInt(e.target.value, 10) })
              }
            />
          </div>
        )}
      </div>

      <div className={styles.layoutSelector}>
        <label className={styles.layoutLabel}>{t('layout')}</label>
        <select
          className={styles.layoutSelect}
          value={layoutId}
          onChange={(e) => onLayoutChange(e.target.value as LayoutId)}
        >
          {LAYOUT_OPTIONS.map((id) => (
            <option key={id} value={id}>{t(`layout.${id}`)}</option>
          ))}
        </select>
      </div>

      <button className={styles.btnRun} onClick={onRun} disabled={running}>
        {running ? t('status.running') : `▶ ${t('run')}`}
      </button>
    </>
  );
}
