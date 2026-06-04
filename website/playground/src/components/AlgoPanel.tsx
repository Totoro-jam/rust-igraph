import type { AlgoId, AlgoParams } from '../types';

const ALGO_LIST: AlgoId[] = [
  'pagerank',
  'louvain',
  'betweenness',
  'closeness',
  'eigenvector',
  'bfs',
  'dfs',
  'components',
  'infomap',
  'spinglass',
  'label_propagation',
  'walktrap',
  'leiden',
  'fast_greedy',
  'leading_eigenvector',
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
      <div className="algo-list">
        {ALGO_LIST.map((id) => (
          <label key={id} className={`algo-option ${algo === id ? 'active' : ''}`}>
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

      <div className="algo-params">
        {algo === 'pagerank' && (
          <div className="param-row">
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
        {(algo === 'bfs' || algo === 'dfs') && (
          <div className="param-row">
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
      </div>

      <button className="btn-run" onClick={onRun} disabled={running}>
        {running ? t('status.running') : `▶ ${t('run')}`}
      </button>
    </>
  );
}
