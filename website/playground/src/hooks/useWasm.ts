import { useState, useCallback, useRef, useEffect } from 'react';
import type { AlgoId, AlgoParams, Edge, RunResult, WorkerResponse } from '../types';
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
  'bfs', 'dfs', 'components', 'infomap', 'spinglass',
]);

export function useWasm(onResult: (result: RunResult) => void) {
  const [status, setStatus] = useState<WasmStatus>('loading');
  const [wasmAvailable, setWasmAvailable] = useState(false);
  const workerRef = useRef<Worker | null>(null);
  const onResultRef = useRef(onResult);
  onResultRef.current = onResult;

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

  return { status, wasmAvailable, run } as const;
}
