import type { WorkerRequest, WorkerResponse, AlgoId, AlgoResult, Edge, AlgoParams, LayoutId, GeneratorId, GeneratorParams, GeneratedGraph } from './types';

let WasmGraph: {
  fromEdges(edges: Uint32Array, directed: boolean): WasmGraphInstance;
  erdosRenyi(n: number, p: number, seed: bigint): WasmGraphInstance;
  fullGraph(n: number): WasmGraphInstance;
  cycleGraph(n: number): WasmGraphInstance;
  ringGraph(n: number, circular: boolean): WasmGraphInstance;
  wattsStrogatz(n: number, k: number, p: number, seed: bigint): WasmGraphInstance;
  barabasiAlbert(n: number, m: number, seed: bigint): WasmGraphInstance;
  pathGraph(n: number, directed: boolean): WasmGraphInstance;
  starGraph(n: number): WasmGraphInstance;
  famousGraph(name: string): WasmGraphInstance;
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
  degreeDistribution(): string;
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
  coreness(): string;
  eccentricity(): string;
  density(): string;
  radius(): string;
  meanDistance(): string;
  meanDegree(): string;
  assortativityDegree(): string;
  constraint(): string;
  reciprocity(): string;
  diameter(): string;
  graphProperties(): string;
  isTree(): string;
  isForest(): string;
  isDag(): string;
  isAcyclic(): string;
  isComplete(): string;
  isBiconnected(): string;
  isTournament(): string;
  isCubic(): string;
  isCycle(): string;
  isPath(): string;
  isStar(): string;
  isWheel(): string;
  isPerfect(): string;
  isTriangleFree(): string;
  isOuterplanar(): string;
  automorphismGroup(): string;
  girth(): string;
  distances(source: number): string;
  floydWarshallDistances(): string;
  fundamentalCycles(): string;
  minimumCycleBasis(): string;
  trussness(): string;
  listTriangles(): string;
  simplify(): WasmGraphInstance;
  lineGraph(): WasmGraphInstance;
  complement(): WasmGraphInstance;
  randomWalk(start: number, steps: number, seed: bigint): string;
  shortestPath(source: number, target: number): string;
  layoutFr(niter: number): string;
  layoutKamadaKawai(): string;
  layoutCircle(): string;
  layoutRandom(seed: number): string;
  layoutGrid(width: number): string;
  layoutStar(center: number): string;
  cliqueNumber(): string;
  independenceNumber(): string;
  maximalCliques(): string;
  vertexConnectivity(): string;
  edgeConnectivity(): string;
  minimumSpanningTree(weights?: number[]): string;
  bellmanFordDistances(source: number, weights: number[]): string;
  strength(weights: number[]): string;
  feedbackArcSet(weights?: number[]): string;
  closenessWeighted(weights: number[]): string;
  betweennessWeighted(weights: number[]): string;
  biconnectedComponents(): string;
  isBipartiteDetailed(): string;
  isEulerian(): string;
  eulerianPath(): string;
  eulerianCycle(): string;
  maximumCut(): string;
  mincutValue(): string;
  vertexDisjointPaths(source: number, target: number): string;
  edgeDisjointPaths(source: number, target: number): string;
  globalEfficiency(): string;
  localEfficiency(): string;
  degeneracy(): string;
  findCycle(): string;
  allSimplePaths(source: number, target: number): string;
  cohesiveBlocks(): string;
  avgNearestNeighborDegree(): string;
  chromaticNumberUpperBound(): string;
  convergenceDegree(): string;
  similarityJaccard(): string;
  similarityDice(): string;
  communityVoronoi(): string;
  graphCenter(): string;
  neighborhood(order: number): string;
  kShortestPaths(source: number, target: number, k: number): string;
  allMinimalStSeparators(): string;
  clusteringCoefficients(): string;
  averagePathLength(): string;
  getEdges(): Uint32Array;
  isDirected(): boolean;
  vcount(): number;
  ecount(): number;
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
      case 'graph_stats': {
        const stats = JSON.parse(graph.graphStats());
        const densityRes = JSON.parse(graph.density());
        const radiusRes = JSON.parse(graph.radius());
        const meanDistRes = JSON.parse(graph.meanDistance());
        const meanDegRes = JSON.parse(graph.meanDegree());
        const assortRes = JSON.parse(graph.assortativityDegree());
        const recipRes = JSON.parse(graph.reciprocity());
        const props = JSON.parse(graph.graphProperties());
        resultJson = JSON.stringify({
          ...stats,
          density: densityRes.density,
          radius: radiusRes.radius,
          mean_distance: meanDistRes.mean_distance,
          mean_degree: meanDegRes.mean_degree,
          assortativity: assortRes.assortativity,
          reciprocity: recipRes.reciprocity,
          properties: props,
        });
        break;
      }
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
      case 'coreness':
        resultJson = graph.coreness();
        break;
      case 'eccentricity':
        resultJson = graph.eccentricity();
        break;
      case 'constraint':
        resultJson = graph.constraint();
        break;
      case 'diameter':
        resultJson = graph.diameter();
        break;
      case 'shortest_path':
        resultJson = graph.shortestPath(params.source ?? 0, params.target ?? 1);
        break;
      case 'random_walk':
        resultJson = graph.randomWalk(params.source ?? 0, 20, BigInt(42));
        break;
      case 'fundamental_cycles':
        resultJson = graph.fundamentalCycles();
        break;
      case 'list_triangles':
        resultJson = graph.listTriangles();
        break;
      case 'girth':
        resultJson = graph.girth();
        break;
      case 'trussness':
        resultJson = graph.trussness();
        break;
      case 'automorphism_group':
        resultJson = graph.automorphismGroup();
        break;
      case 'clique_number':
        resultJson = graph.cliqueNumber();
        break;
      case 'independence_number':
        resultJson = graph.independenceNumber();
        break;
      case 'maximal_cliques':
        resultJson = graph.maximalCliques();
        break;
      case 'vertex_connectivity':
        resultJson = graph.vertexConnectivity();
        break;
      case 'edge_connectivity':
        resultJson = graph.edgeConnectivity();
        break;
      case 'minimum_spanning_tree':
        resultJson = graph.minimumSpanningTree();
        break;
      case 'bellman_ford':
        resultJson = graph.bellmanFordDistances(params?.source ?? 0, Array.from({ length: graph.ecount() }, () => 1));
        break;
      case 'degree_distribution':
        resultJson = graph.degreeDistribution();
        break;
      case 'feedback_arc_set':
        resultJson = graph.feedbackArcSet();
        break;
      case 'minimum_cycle_basis':
        resultJson = graph.minimumCycleBasis();
        break;
      case 'biconnected_components':
        resultJson = graph.biconnectedComponents();
        break;
      case 'bipartite_check':
        resultJson = graph.isBipartiteDetailed();
        break;
      case 'maximum_cut':
        resultJson = graph.maximumCut();
        break;
      case 'global_efficiency':
        resultJson = graph.globalEfficiency();
        break;
      case 'local_efficiency':
        resultJson = graph.localEfficiency();
        break;
      case 'degeneracy':
        resultJson = graph.degeneracy();
        break;
      case 'all_simple_paths':
        resultJson = graph.allSimplePaths(params.source ?? 0, params.target ?? 1);
        break;
      case 'find_cycle':
        resultJson = graph.findCycle();
        break;
      case 'mincut_value':
        resultJson = graph.mincutValue();
        break;
      case 'vertex_disjoint_paths':
        resultJson = graph.vertexDisjointPaths(params.source ?? 0, params.target ?? 1);
        break;
      case 'edge_disjoint_paths':
        resultJson = graph.edgeDisjointPaths(params.source ?? 0, params.target ?? 1);
        break;
      case 'is_eulerian':
        resultJson = graph.isEulerian();
        break;
      case 'cohesive_blocks':
        resultJson = graph.cohesiveBlocks();
        break;
      case 'avg_nearest_neighbor_degree':
        resultJson = graph.avgNearestNeighborDegree();
        break;
      case 'chromatic_number':
        resultJson = graph.chromaticNumberUpperBound();
        break;
      case 'convergence_degree':
        resultJson = graph.convergenceDegree();
        break;
      case 'similarity_jaccard':
        resultJson = graph.similarityJaccard();
        break;
      case 'community_voronoi':
        resultJson = graph.communityVoronoi();
        break;
      case 'graph_center':
        resultJson = graph.graphCenter();
        break;
      case 'clustering_coefficients':
        resultJson = graph.clusteringCoefficients();
        break;
      case 'average_path_length':
        resultJson = graph.averagePathLength();
        break;
      case 'k_shortest_paths':
        resultJson = graph.kShortestPaths(params.source ?? 0, params.target ?? (graph.vcount() > 1 ? graph.vcount() - 1 : 0), 5);
        break;
      case 'graph_properties':
        resultJson = graph.graphProperties();
        break;
      case 'similarity_dice':
        resultJson = graph.similarityDice();
        break;
      case 'assortativity_degree': {
        const raw = JSON.parse(graph.assortativityDegree());
        resultJson = JSON.stringify({ value: raw.assortativity ?? 0 });
        break;
      }
      case 'density': {
        const raw = JSON.parse(graph.density());
        resultJson = JSON.stringify({ value: raw.density ?? 0 });
        break;
      }
      case 'radius': {
        const raw = JSON.parse(graph.radius());
        resultJson = JSON.stringify({ value: raw.radius ?? 0 });
        break;
      }
      case 'mean_degree': {
        const raw = JSON.parse(graph.meanDegree());
        resultJson = JSON.stringify({ value: raw.mean_degree ?? 0 });
        break;
      }
      case 'mean_distance': {
        const raw = JSON.parse(graph.meanDistance());
        resultJson = JSON.stringify({ value: raw.mean_distance ?? 0 });
        break;
      }
      case 'reciprocity': {
        const raw = JSON.parse(graph.reciprocity());
        resultJson = JSON.stringify({ value: raw.reciprocity ?? 0 });
        break;
      }
      case 'neighborhood':
        resultJson = graph.neighborhood(1);
        break;
      case 'all_minimal_st_separators':
        resultJson = graph.allMinimalStSeparators();
        break;
      case 'strength': {
        const w = new Array(graph.ecount()).fill(1.0);
        resultJson = graph.strength(w);
        break;
      }
      default:
        throw new Error(`Algorithm "${algo}" not available in WASM mode`);
    }

    const result = JSON.parse(resultJson) as AlgoResult;
    return { result, coords };
  } finally {
    graph.free();
  }
}

function generateGraph(generator: GeneratorId, params: GeneratorParams): GeneratedGraph {
  if (!WasmGraph) throw new Error('WASM not loaded');

  const seed = BigInt(params.seed ?? 42);
  let graph: WasmGraphInstance;

  switch (generator) {
    case 'erdos_renyi':
      graph = WasmGraph.erdosRenyi(params.n ?? 50, params.p ?? 0.1, seed);
      break;
    case 'barabasi_albert':
      graph = WasmGraph.barabasiAlbert(params.n ?? 50, params.m ?? 2, seed);
      break;
    case 'watts_strogatz':
      graph = WasmGraph.wattsStrogatz(params.n ?? 30, params.k ?? 4, params.p ?? 0.1, seed);
      break;
    case 'complete':
      graph = WasmGraph.fullGraph(params.n ?? 10);
      break;
    case 'cycle':
      graph = WasmGraph.cycleGraph(params.n ?? 20);
      break;
    case 'path':
      graph = WasmGraph.pathGraph(params.n ?? 10, params.directed ?? false);
      break;
    case 'star':
      graph = WasmGraph.starGraph(params.n ?? 12);
      break;
    case 'ring':
      graph = WasmGraph.ringGraph(params.n ?? 20, params.circular ?? true);
      break;
    case 'famous':
      graph = WasmGraph.famousGraph(params.name ?? 'Petersen');
      break;
    default:
      throw new Error(`Unknown generator: ${generator}`);
  }

  try {
    const flatEdges = graph.getEdges();
    const directed = graph.isDirected();
    const vcount = graph.vcount();
    const edges: Edge[] = [];
    for (let i = 0; i < flatEdges.length; i += 2) {
      edges.push([flatEdges[i]!, flatEdges[i + 1]!]);
    }
    return { edges, directed, vcount };
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

    case 'generate': {
      if (!wasmReady) {
        post({ type: 'error', message: 'WASM not available' });
        return;
      }

      try {
        const data = generateGraph(msg.generator, msg.params);
        post({ type: 'generated', data });
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
