import type { AlgoId, AlgoParams } from '../../types';
import styles from './index.module.css';

interface AlgoCategory {
  key: string;
  algos: AlgoId[];
}

const ALGO_CATEGORIES: AlgoCategory[] = [
  {
    key: 'centrality',
    algos: ['pagerank', 'betweenness', 'closeness', 'eigenvector', 'harmonic', 'hits', 'katz'],
  },
  {
    key: 'community',
    algos: [
      'louvain', 'leiden', 'infomap', 'label_propagation', 'walktrap',
      'fast_greedy', 'leading_eigenvector', 'edge_betweenness', 'spinglass', 'fluid',
    ],
  },
  {
    key: 'traversal',
    algos: ['bfs', 'dfs', 'dijkstra', 'max_flow'],
  },
  {
    key: 'structure',
    algos: ['components', 'graph_stats', 'articulation_points', 'degree_sequence'],
  },
];

interface AlgoPanelProps {
  algo: AlgoId;
  params: AlgoParams;
  running: boolean;
  onAlgoChange: (algo: AlgoId) => void;
  onParamsChange: (params: AlgoParams) => void;
  onRun: () => void;
  t: (key: string) => string;
}

export function AlgoPanel({
  algo,
  params,
  running,
  onAlgoChange,
  onParamsChange,
  onRun,
  t,
}: AlgoPanelProps) {
  return (
    <>
      <div className={styles.algoList}>
        {ALGO_CATEGORIES.map((cat) => (
          <div key={cat.key} className={styles.algoGroup}>
            <div className={styles.groupHeader}>{t(`cat.${cat.key}`)}</div>
            {cat.algos.map((id) => (
              <label key={id} className={`${styles.algoOption} ${algo === id ? styles.active : ''}`}>
                <input
                  type="radio"
                  name="algo"
                  value={id}
                  checked={algo === id}
                  onChange={() => onAlgoChange(id)}
                />
                <span>{t(`algo.${id}`)}</span>
              </label>
            ))}
          </div>
        ))}
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
        {(algo === 'bfs' || algo === 'dfs' || algo === 'dijkstra' || algo === 'max_flow') && (
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
        {algo === 'max_flow' && (
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

      <button className={styles.btnRun} onClick={onRun} disabled={running}>
        {running ? t('status.running') : `▶ ${t('run')}`}
      </button>
    </>
  );
}
