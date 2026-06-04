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
  | 'degree_sequence';

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

export type AlgoResult =
  | AlgoResultScores
  | AlgoResultMembership
  | AlgoResultOrder
  | AlgoResultHits
  | AlgoResultStats
  | AlgoResultScalar
  | AlgoResultVertices
  | AlgoResultDistances
  | AlgoResultDegrees;

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
  | { type: 'run'; algo: AlgoId; edges: Edge[]; directed: boolean; params: AlgoParams }
  | { type: 'cancel' };

export type WorkerResponse =
  | { type: 'ready'; wasmAvailable: boolean }
  | { type: 'result'; data: RunResult }
  | { type: 'error'; message: string }
  | { type: 'progress'; percent: number };
