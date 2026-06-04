import type { WorkerRequest, WorkerResponse, AlgoId, AlgoResult, Edge, AlgoParams, LayoutId } from './types';

let WasmGraph: {
  fromEdges(edges: Uint32Array, directed: boolean): WasmGraphInstance;
} | null = null;

interface WasmGraphInstance {
  bfs(root: number): string;
  dfs(root: number): string;
  dijkstra(source: number, weights: Float64Array): string;
  pagerank(): string;
  louvain(): string;
  betweenness(): string;
  closeness(): string;
  eigenvectorCentrality(): string;
  connectedComponents(): string;
  infomap(): string;
  spinglass(): string;
  labelPropagation(): string;
  walktrap(): string;
  leiden(): string;
  fastGreedy(): string;
  leadingEigenvector(): string;
  edgeBetweennessCommunity(): string;
  fluidCommunities(k: number): string;
  harmonicCentrality(): string;
  hubAndAuthorityScores(): string;
  katzCentrality(): string;
  graphStats(): string;
  maxFlow(source: number, target: number): string;
  articulationPoints(): string;
  degreeSequence(): string;
  stronglyConnectedComponents(): string;
  bridges(): string;
  vertexColoring(): string;
  topologicalSort(): string;
  transitivity(): string;
  edgeBetweenness(): string;
  triadCensus(): string;
  canonicalPermutation(): string;
  countAutomorphisms(): string;
  isomorphicBliss(other: WasmGraphInstance): string;
  layoutFr(niter: number): string;
  layoutKamadaKawai(): string;
  layoutCircle(): string;
  layoutRandom(seed: number): string;
  layoutGrid(width: number): string;
  layoutStar(center: number): string;
  free(): void;
}

async function initWasm(): Promise<boolean> {
  try {
    const workerUrl = self.location.href;
    const playgroundRoot = workerUrl.includes('/assets/')
      ? workerUrl.replace(/\/assets\/[^/]*$/, '')
      : workerUrl.replace(/\/[^/]*$/, '');
    const wasmModule = await import(/* @vite-ignore */ `${playgroundRoot}/wasm/igraph_wasm.js`);
    await wasmModule.default();
    WasmGraph = wasmModule.WasmGraph;
    return true;
  } catch {
    return false;
  }
}

function flattenEdges(edges: Edge[]): Uint32Array {
  const flat = new Uint32Array(edges.length * 2);
  for (let i = 0; i < edges.length; i++) {
    flat[i * 2] = edges[i]![0];
    flat[i * 2 + 1] = edges[i]![1];
  }
  return flat;
}

function computeLayout(graph: WasmGraphInstance, layoutId: LayoutId): [number, number][] {
  let json: string;
  switch (layoutId) {
    case 'kamada_kawai':
      json = graph.layoutKamadaKawai();
      break;
    case 'circle':
      json = graph.layoutCircle();
      break;
    case 'random':
      json = graph.layoutRandom(42);
      break;
    case 'grid':
      json = graph.layoutGrid(0);
      break;
    case 'star':
      json = graph.layoutStar(0);
      break;
    case 'fr':
    default:
      json = graph.layoutFr(300);
      break;
  }
  return (JSON.parse(json) as { coords: [number, number][] }).coords;
}

function runWasm(
  algo: AlgoId,
  edges: Edge[],
  directed: boolean,
  params: AlgoParams,
  layoutId: LayoutId,
): { result: AlgoResult; coords: [number, number][] } {
  if (!WasmGraph) throw new Error('WASM not loaded');

  const flat = flattenEdges(edges);
  const graph = WasmGraph.fromEdges(flat, directed);

  try {
    const coords = computeLayout(graph, layoutId);

    let resultJson: string;
    switch (algo) {
      case 'pagerank':
        resultJson = graph.pagerank();
        break;
      case 'louvain':
        resultJson = graph.louvain();
        break;
      case 'betweenness':
        resultJson = graph.betweenness();
        break;
      case 'bfs':
        resultJson = graph.bfs(params.source ?? 0);
        break;
      case 'dfs':
        resultJson = graph.dfs(params.source ?? 0);
        break;
      case 'closeness':
        resultJson = graph.closeness();
        break;
      case 'eigenvector':
        resultJson = graph.eigenvectorCentrality();
        break;
      case 'components':
        resultJson = graph.connectedComponents();
        break;
      case 'infomap':
        resultJson = graph.infomap();
        break;
      case 'spinglass':
        resultJson = graph.spinglass();
        break;
      case 'label_propagation':
        resultJson = graph.labelPropagation();
        break;
      case 'walktrap':
        resultJson = graph.walktrap();
        break;
      case 'leiden':
        resultJson = graph.leiden();
        break;
      case 'fast_greedy':
        resultJson = graph.fastGreedy();
        break;
      case 'leading_eigenvector':
        resultJson = graph.leadingEigenvector();
        break;
      case 'edge_betweenness':
        resultJson = graph.edgeBetweennessCommunity();
        break;
      case 'fluid':
        resultJson = graph.fluidCommunities(params.source ?? 3);
        break;
      case 'harmonic':
        resultJson = graph.harmonicCentrality();
        break;
      case 'hits':
        resultJson = graph.hubAndAuthorityScores();
        break;
      case 'katz':
        resultJson = graph.katzCentrality();
        break;
      case 'dijkstra': {
        const weights = new Float64Array(edges.length).fill(1.0);
        resultJson = graph.dijkstra(params.source ?? 0, weights);
        break;
      }
      case 'graph_stats':
        resultJson = graph.graphStats();
        break;
      case 'max_flow':
        resultJson = graph.maxFlow(params.source ?? 0, params.target ?? 1);
        break;
      case 'articulation_points':
        resultJson = graph.articulationPoints();
        break;
      case 'degree_sequence':
        resultJson = graph.degreeSequence();
        break;
      case 'scc':
        resultJson = graph.stronglyConnectedComponents();
        break;
      case 'bridges':
        resultJson = graph.bridges();
        break;
      case 'coloring':
        resultJson = graph.vertexColoring();
        break;
      case 'topological_sort':
        resultJson = graph.topologicalSort();
        break;
      case 'transitivity':
        resultJson = graph.transitivity();
        break;
      case 'edge_betweenness_centrality':
        resultJson = graph.edgeBetweenness();
        break;
      case 'triad_census':
        resultJson = graph.triadCensus();
        break;
      case 'canonical_permutation':
        resultJson = graph.canonicalPermutation();
        break;
      case 'count_automorphisms':
        resultJson = graph.countAutomorphisms();
        break;
      case 'isomorphism':
        resultJson = graph.canonicalPermutation();
        break;
      default:
        throw new Error(`Algorithm "${algo}" not available in WASM mode`);
    }

    const result = JSON.parse(resultJson) as AlgoResult;
    return { result, coords };
  } finally {
    graph.free();
  }
}

function post(msg: WorkerResponse) {
  self.postMessage(msg);
}

let wasmReady = false;

self.onmessage = async (e: MessageEvent<WorkerRequest>) => {
  const msg = e.data;

  switch (msg.type) {
    case 'init': {
      wasmReady = await initWasm();
      post({ type: 'ready', wasmAvailable: wasmReady });
      break;
    }

    case 'run': {
      if (!wasmReady) {
        post({ type: 'error', message: 'WASM not available' });
        return;
      }

      try {
        const t0 = performance.now();
        const { result, coords } = runWasm(msg.algo, msg.edges, msg.directed, msg.params, msg.layout);
        const elapsed_ms = performance.now() - t0;

        post({
          type: 'result',
          data: { algo: msg.algo, result, coords, elapsed_ms },
        });
      } catch (err) {
        post({
          type: 'error',
          message: err instanceof Error ? err.message : String(err),
        });
      }
      break;
    }

    case 'cancel':
      break;
  }
};
