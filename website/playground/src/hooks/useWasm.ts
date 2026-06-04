import { useState, useCallback, useRef, useEffect } from 'react';
import type { AlgoId, AlgoParams, Edge, RunResult } from '../types';
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

export function useWasm() {
  const [status, setStatus] = useState<WasmStatus>('loading');
  const [wasmAvailable, setWasmAvailable] = useState(false);
  const workerRef = useRef<Worker | null>(null);

  useEffect(() => {
    // WASM worker not yet implemented — go straight to demo mode
    setWasmAvailable(false);
    setStatus('ready');

    return () => {
      workerRef.current?.terminate();
    };
  }, []);

  const run = useCallback(
    (
      algo: AlgoId,
      edges: Edge[],
      _directed: boolean,
      params: AlgoParams,
    ): RunResult | null => {
      const vcount = getVcount(edges);
      if (vcount === 0) return null;

      setStatus('running');
      const t0 = performance.now();

      try {
        const coords = layoutFR(vcount, edges, 300);
        const result = runDemoAlgo(algo, vcount, edges, {
          damping: params.damping,
        });
        const elapsed_ms = performance.now() - t0;

        setStatus(wasmAvailable ? 'ready' : 'ready');
        return { algo, result, coords, elapsed_ms };
      } catch {
        setStatus('error');
        return null;
      }
    },
    [wasmAvailable],
  );

  return { status, wasmAvailable, run } as const;
}
