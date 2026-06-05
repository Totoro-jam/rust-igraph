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
  | 'girth'
  | 'shortest_path'
  | 'random_walk'
  | 'fundamental_cycles'
  | 'list_triangles'
  | 'trussness'
  | 'automorphism_group'
  | 'clique_number'
  | 'independence_number'
  | 'maximal_cliques'
  | 'vertex_connectivity'
  | 'edge_connectivity'
  | 'minimum_spanning_tree'
  | 'bellman_ford'
  | 'degree_distribution'
  | 'feedback_arc_set'
  | 'minimum_cycle_basis'
  | 'biconnected_components'
  | 'bipartite_check'
  | 'maximum_cut'
  | 'global_efficiency'
  | 'local_efficiency'
  | 'degeneracy'
  | 'all_simple_paths'
  | 'find_cycle'
  | 'mincut_value'
  | 'vertex_disjoint_paths'
  | 'edge_disjoint_paths'
  | 'is_eulerian'
  | 'cohesive_blocks'
  | 'avg_nearest_neighbor_degree'
  | 'chromatic_number'
  | 'convergence_degree'
  | 'similarity_jaccard'
  | 'community_voronoi'
  | 'graph_center'
  | 'clustering_coefficients'
  | 'average_path_length'
  | 'k_shortest_paths'
  | 'graph_properties'
  | 'similarity_dice'
  | 'assortativity_degree'
  | 'density'
  | 'radius'
  | 'mean_degree'
  | 'mean_distance'
  | 'reciprocity'
  | 'neighborhood'
  | 'all_minimal_st_separators'
  | 'strength';

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

export interface AlgoResultCycles {
  cycles: number[][];
  count: number;
}

export interface AlgoResultTriangles {
  triangles: [number, number, number][];
  count: number;
}

export interface AlgoResultTrussness {
  trussness: number[];
}

export interface AlgoResultAutomorphismGroup {
  generators: number[][];
  count: number;
}

export interface AlgoResultCliques {
  cliques: number[][];
  count: number;
}

export interface AlgoResultMst {
  edges: number[];
  count: number;
}

export interface AlgoResultWeightedDistances {
  distances: (number | null)[];
}

export interface AlgoResultFeedbackArcSet {
  edges: number[];
  count: number;
}

export interface AlgoResultBiconnected {
  count: number;
  components: number[][];
}

export interface AlgoResultBipartiteCheck {
  is_bipartite: boolean;
  types: number[];
}

export interface AlgoResultMaxCut {
  partition: boolean[];
  cut_value: number;
}

export interface AlgoResultEulerian {
  has_path: boolean;
  has_cycle: boolean;
}

export interface AlgoResultSimplePaths {
  paths: number[][];
  count: number;
}

export interface AlgoResultFindCycle {
  vertices: number[];
  edges: number[];
  found: boolean;
}

export interface AlgoResultCohesiveBlocks {
  blocks: number[][];
  cohesion: number[];
  count: number;
}

export interface AlgoResultKnn {
  scores: (number | null)[];
}

export interface AlgoResultSimilarityMatrix {
  matrix: number[][];
  size: number;
}

export interface AlgoResultVoronoi {
  membership: number[];
  generators: number[];
  modularity: number | null;
}

export interface AlgoResultGraphCenter {
  vertices: number[];
  count: number;
}

export interface AlgoResultClusteringCoeff {
  scores: (number | null)[];
}

export interface AlgoResultKPaths {
  paths: { vertices: number[]; weight: number }[];
  count: number;
}

export interface AlgoResultSeparators {
  separators: number[][];
  count: number;
}

export interface AlgoResultGraphProperties {
  is_tree: boolean;
  is_forest: boolean;
  is_dag: boolean;
  is_acyclic: boolean;
  is_complete: boolean;
  is_biconnected: boolean;
  is_bipartite: boolean;
  is_connected: boolean;
  is_tournament: boolean;
  is_cubic: boolean;
  is_cycle: boolean;
  is_path: boolean;
  is_star: boolean;
  is_wheel: boolean;
  is_perfect: boolean;
  is_triangle_free: boolean;
  is_outerplanar: boolean;
}

export interface AlgoResultNeighborhood {
  neighborhoods: number[][];
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
  | AlgoResultDiameter
  | AlgoResultCycles
  | AlgoResultTriangles
  | AlgoResultTrussness
  | AlgoResultAutomorphismGroup
  | AlgoResultCliques
  | AlgoResultMst
  | AlgoResultWeightedDistances
  | AlgoResultFeedbackArcSet
  | AlgoResultBiconnected
  | AlgoResultBipartiteCheck
  | AlgoResultMaxCut
  | AlgoResultEulerian
  | AlgoResultSimplePaths
  | AlgoResultFindCycle
  | AlgoResultCohesiveBlocks
  | AlgoResultKnn
  | AlgoResultSimilarityMatrix
  | AlgoResultVoronoi
  | AlgoResultGraphCenter
  | AlgoResultClusteringCoeff
  | AlgoResultKPaths
  | AlgoResultSeparators
  | AlgoResultGraphProperties
  | AlgoResultNeighborhood;

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

export type GeneratorId =
  | 'erdos_renyi'
  | 'barabasi_albert'
  | 'watts_strogatz'
  | 'complete'
  | 'cycle'
  | 'path'
  | 'star'
  | 'ring'
  | 'famous';

export interface GeneratorParams {
  n?: number;
  p?: number;
  m?: number;
  k?: number;
  seed?: number;
  directed?: boolean;
  circular?: boolean;
  name?: string;
}

export interface GeneratedGraph {
  edges: Edge[];
  directed: boolean;
  vcount: number;
}

export type WorkerRequest =
  | { type: 'init' }
  | { type: 'run'; algo: AlgoId; edges: Edge[]; directed: boolean; params: AlgoParams; layout: LayoutId }
  | { type: 'generate'; generator: GeneratorId; params: GeneratorParams }
  | { type: 'cancel' };

export type WorkerResponse =
  | { type: 'ready'; wasmAvailable: boolean }
  | { type: 'result'; data: RunResult }
  | { type: 'generated'; data: GeneratedGraph }
  | { type: 'error'; message: string }
  | { type: 'progress'; percent: number };
