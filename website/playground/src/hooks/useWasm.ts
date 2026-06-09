import { useState, useCallback, useRef, useEffect } from 'react';
import type { AlgoId, AlgoParams, Edge, GeneratorId, GeneratorParams, GeneratedGraph, LayoutId, RunResult, WorkerResponse } from '../types';
import { runDemoAlgo, layoutFR } from '../algorithms';

type WasmStatus = 'loading' | 'ready' | 'error' | 'running';

function getVcount(edges: Edge[]): number {
  let max = -1;
  for (const [u, v] of edges) {
    if (u > max) max = u;
    if (v > max) max = v;
  }
  return max + 1;
}

const WASM_SUPPORTED_ALGOS: Set<AlgoId> = new Set([
  'pagerank', 'louvain', 'betweenness', 'closeness', 'eigenvector',
  'bfs', 'dfs', 'dijkstra', 'components', 'infomap', 'spinglass',
  'label_propagation', 'walktrap', 'leiden', 'fast_greedy', 'leading_eigenvector',
  'edge_betweenness', 'fluid', 'harmonic', 'hits', 'katz',
  'graph_stats', 'max_flow', 'articulation_points', 'degree_sequence',
  'scc', 'bridges', 'coloring', 'topological_sort', 'transitivity',
  'edge_betweenness_centrality', 'triad_census',
  'canonical_permutation', 'count_automorphisms', 'isomorphism',
  'coreness', 'eccentricity', 'constraint', 'diameter', 'shortest_path',
  'random_walk', 'fundamental_cycles', 'list_triangles', 'girth',
  'trussness', 'automorphism_group',
  'clique_number', 'independence_number', 'maximal_cliques',
  'vertex_connectivity', 'edge_connectivity', 'minimum_spanning_tree',
  'bellman_ford', 'k_shortest_paths',
]);

export function useWasm(
  onResult: (result: RunResult) => void,
  onGenerated?: (data: GeneratedGraph) => void,
) {
  const [status, setStatus] = useState<WasmStatus>('loading');
  const [wasmAvailable, setWasmAvailable] = useState(false);
  const workerRef = useRef<Worker | null>(null);
  const onResultRef = useRef(onResult);
  onResultRef.current = onResult;
  const onGeneratedRef = useRef(onGenerated);
  onGeneratedRef.current = onGenerated;

  const pendingRunRef = useRef<{
    algo: AlgoId;
    edges: Edge[];
    directed: boolean;
    params: AlgoParams;
  } | null>(null);

  const runDemoFallback = useCallback(
    (algo: AlgoId, edges: Edge[], _directed: boolean, params: AlgoParams): RunResult | null => {
      const vcount = getVcount(edges);
      if (vcount === 0) return null;
      const t0 = performance.now();
      try {
        const coords = layoutFR(vcount, edges, 300);
        const result = runDemoAlgo(algo, vcount, edges, {
          damping: params.damping,
          source: params.source,
          target: params.target,
        });
        const elapsed_ms = performance.now() - t0;
        return { algo, result, coords, elapsed_ms };
      } catch {
        return null;
      }
    },
    [],
  );

  useEffect(() => {
    const worker = new Worker(
      new URL('../worker.ts', import.meta.url),
      { type: 'module' },
    );

    worker.onmessage = (e: MessageEvent<WorkerResponse>) => {
      const msg = e.data;
      switch (msg.type) {
        case 'ready':
          setWasmAvailable(msg.wasmAvailable);
          setStatus('ready');
          break;
        case 'result':
          setStatus('ready');
          pendingRunRef.current = null;
          onResultRef.current(msg.data);
          break;
        case 'generated':
          setStatus('ready');
          onGeneratedRef.current?.(msg.data);
          break;
        case 'error': {
          setStatus('ready');
          const pending = pendingRunRef.current;
          pendingRunRef.current = null;
          if (pending) {
            const fallback = runDemoFallback(pending.algo, pending.edges, pending.directed, pending.params);
            if (fallback) onResultRef.current(fallback);
          }
          break;
        }
      }
    };

    worker.onerror = () => {
      setWasmAvailable(false);
      setStatus('ready');
    };

    worker.postMessage({ type: 'init' });
    workerRef.current = worker;

    const timeout = setTimeout(() => {
      setStatus((prev) => (prev === 'loading' ? 'ready' : prev));
    }, 3000);

    return () => {
      clearTimeout(timeout);
      worker.terminate();
    };
  }, [runDemoFallback]);

  const run = useCallback(
    (
      algo: AlgoId,
      edges: Edge[],
      directed: boolean,
      params: AlgoParams,
      layout: LayoutId = 'fr',
    ): RunResult | null => {
      const vcount = getVcount(edges);
      if (vcount === 0) return null;

      if (wasmAvailable && WASM_SUPPORTED_ALGOS.has(algo) && workerRef.current) {
        setStatus('running');
        pendingRunRef.current = { algo, edges, directed, params };
        workerRef.current.postMessage({
          type: 'run',
          algo,
          edges,
          directed,
          params,
          layout,
        });
        return null;
      }

      const result = runDemoFallback(algo, edges, directed, params);
      if (result) {
        return result;
      }

      setStatus('error');
      return null;
    },
    [wasmAvailable, runDemoFallback],
  );

  const generate = useCallback(
    (generator: GeneratorId, params: GeneratorParams) => {
      if (!wasmAvailable || !workerRef.current) return;
      setStatus('running');
      workerRef.current.postMessage({ type: 'generate', generator, params });
    },
    [wasmAvailable],
  );

  return { status, wasmAvailable, run, generate } as const;
}
