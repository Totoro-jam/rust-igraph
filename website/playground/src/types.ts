export type Edge = [number, number];

export interface GraphData {
  edges: Edge[];
  vcount: number;
  directed: boolean;
}

export type AlgoId =
  | 'pagerank'
  | 'louvain'
  | 'betweenness'
  | 'closeness'
  | 'eigenvector'
  | 'bfs'
  | 'dfs'
  | 'dijkstra'
  | 'components'
  | 'infomap'
  | 'spinglass'
  | 'label_propagation'
  | 'walktrap'
  | 'leiden'
  | 'fast_greedy'
  | 'leading_eigenvector'
  | 'edge_betweenness'
  | 'fluid'
  | 'harmonic'
  | 'hits'
  | 'katz'
  | 'graph_stats'
  | 'max_flow'
  | 'articulation_points'
  | 'degree_sequence'
  | 'scc'
  | 'bridges'
  | 'coloring'
  | 'topological_sort'
  | 'transitivity'
  | 'edge_betweenness_centrality'
  | 'triad_census'
  | 'canonical_permutation'
  | 'count_automorphisms'
  | 'isomorphism'
  | 'coreness'
  | 'eccentricity'
  | 'constraint'
  | 'diameter'
  | 'shortest_path'
  | 'random_walk';

export type LayoutId = 'fr' | 'kamada_kawai' | 'circle' | 'random' | 'grid' | 'star';

export interface AlgoParams {
  source?: number;
  target?: number;
  damping?: number;
}

export interface AlgoResultScores {
  scores: number[];
}

export interface AlgoResultMembership {
  membership: number[];
  modularity?: number;
  codelength?: number;
  count?: number;
  nb_clusters?: number;
  quality?: number;
}

export interface AlgoResultOrder {
  order: number[];
}

export interface AlgoResultHits {
  hub: number[];
  authority: number[];
}

export interface AlgoResultStats {
  vcount: number;
  ecount: number;
  is_directed: boolean;
  is_connected: boolean;
  diameter: number;
  girth: number;
  triangles: number;
  is_bipartite: boolean;
  density?: number | null;
  radius?: number | null;
  mean_distance?: number | null;
  mean_degree?: number | null;
  assortativity?: number | null;
  reciprocity?: number | null;
}

export interface AlgoResultScalar {
  value: number;
}

export interface AlgoResultVertices {
  vertices: number[];
}

export interface AlgoResultDistances {
  distances: number[];
}

export interface AlgoResultDegrees {
  degrees: number[];
}

export interface AlgoResultScc {
  membership: number[];
  count: number;
}

export interface AlgoResultBridges {
  edges: [number, number][];
  count: number;
}

export interface AlgoResultColoring {
  colors: number[];
  chromatic: number;
}

export interface AlgoResultTransitivity {
  value: number;
}

export interface AlgoResultEdgeBetweenness {
  scores: number[];
}

export interface AlgoResultTriadCensus {
  counts: number[];
}

export interface AlgoResultPermutation {
  permutation: number[];
}

export interface AlgoResultAutomorphisms {
  count: number;
}

export interface AlgoResultIsomorphism {
  isomorphic: boolean;
  mapping: number[];
}

export interface AlgoResultCores {
  cores: number[];
}

export interface AlgoResultValues {
  values: number[];
}

export interface AlgoResultPath {
  path: number[];
}

export interface AlgoResultWalk {
  vertices: number[];
}

export interface AlgoResultDiameter {
  diameter: number | null;
}

export type AlgoResult =
  | AlgoResultScores
  | AlgoResultMembership
  | AlgoResultOrder
  | AlgoResultHits
  | AlgoResultStats
  | AlgoResultScalar
  | AlgoResultVertices
  | AlgoResultDistances
  | AlgoResultDegrees
  | AlgoResultBridges
  | AlgoResultColoring
  | AlgoResultTransitivity
  | AlgoResultEdgeBetweenness
  | AlgoResultTriadCensus
  | AlgoResultPermutation
  | AlgoResultAutomorphisms
  | AlgoResultIsomorphism
  | AlgoResultCores
  | AlgoResultValues
  | AlgoResultPath
  | AlgoResultWalk
  | AlgoResultDiameter;

export interface RunResult {
  algo: AlgoId;
  result: AlgoResult;
  coords: [number, number][];
  elapsed_ms: number;
}

export interface PresetGraph {
  id: string;
  edges: Edge[];
  directed: boolean;
}

export type WorkerRequest =
  | { type: 'init' }
  | { type: 'run'; algo: AlgoId; edges: Edge[]; directed: boolean; params: AlgoParams; layout: LayoutId }
  | { type: 'cancel' };

export type WorkerResponse =
  | { type: 'ready'; wasmAvailable: boolean }
  | { type: 'result'; data: RunResult }
  | { type: 'error'; message: string }
  | { type: 'progress'; percent: number };
