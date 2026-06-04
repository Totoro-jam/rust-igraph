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
  | 'components'
  | 'infomap'
  | 'spinglass'
  | 'label_propagation'
  | 'walktrap'
  | 'leiden'
  | 'fast_greedy'
  | 'leading_eigenvector'
  | 'edge_betweenness'
  | 'fluid';

export interface AlgoParams {
  source?: number;
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

export type AlgoResult = AlgoResultScores | AlgoResultMembership | AlgoResultOrder;

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
