import { useEffect, useRef } from 'react';
import type { AlgoId, AlgoParams } from '../types';
import { PRESETS } from '../presets';

const VALID_ALGOS = new Set<string>([
  'pagerank', 'louvain', 'betweenness', 'closeness', 'eigenvector',
  'bfs', 'dfs', 'dijkstra', 'components', 'infomap', 'spinglass',
  'label_propagation', 'walktrap', 'leiden', 'fast_greedy',
  'leading_eigenvector', 'edge_betweenness', 'fluid', 'harmonic',
  'hits', 'katz', 'graph_stats', 'max_flow', 'articulation_points',
  'degree_sequence',
]);

export interface UrlState {
  preset?: string;
  algo?: AlgoId;
  directed?: boolean;
  damping?: number;
  source?: number;
  target?: number;
}

export function readUrlState(): UrlState {
  const params = new URLSearchParams(window.location.search);
  const state: UrlState = {};

  const preset = params.get('preset');
  if (preset && preset in PRESETS) {
    state.preset = preset;
  }

  const algo = params.get('algo');
  if (algo && VALID_ALGOS.has(algo)) {
    state.algo = algo as AlgoId;
  }

  const directed = params.get('directed');
  if (directed === '1' || directed === 'true') {
    state.directed = true;
  } else if (directed === '0' || directed === 'false') {
    state.directed = false;
  }

  const damping = params.get('damping');
  if (damping !== null) {
    const v = parseFloat(damping);
    if (Number.isFinite(v) && v >= 0 && v <= 1) state.damping = v;
  }

  const source = params.get('source');
  if (source !== null) {
    const v = parseInt(source, 10);
    if (Number.isFinite(v) && v >= 0) state.source = v;
  }

  const target = params.get('target');
  if (target !== null) {
    const v = parseInt(target, 10);
    if (Number.isFinite(v) && v >= 0) state.target = v;
  }

  return state;
}

export function useUrlSync(
  preset: string,
  algo: AlgoId,
  directed: boolean,
  params: AlgoParams,
): void {
  const isFirstRender = useRef(true);

  useEffect(() => {
    if (isFirstRender.current) {
      isFirstRender.current = false;
      return;
    }

    const sp = new URLSearchParams();
    if (preset !== 'karate') sp.set('preset', preset);
    if (algo !== 'pagerank') sp.set('algo', algo);
    if (directed) sp.set('directed', '1');
    if (algo === 'pagerank' && params.damping !== undefined && params.damping !== 0.85) {
      sp.set('damping', String(params.damping));
    }
    if ((algo === 'bfs' || algo === 'dfs' || algo === 'dijkstra' || algo === 'max_flow')
        && params.source !== undefined && params.source !== 0) {
      sp.set('source', String(params.source));
    }
    if (algo === 'max_flow' && params.target !== undefined && params.target !== 1) {
      sp.set('target', String(params.target));
    }

    const qs = sp.toString();
    const newUrl = qs ? `${window.location.pathname}?${qs}` : window.location.pathname;
    window.history.replaceState(null, '', newUrl);
  }, [preset, algo, directed, params]);
}
