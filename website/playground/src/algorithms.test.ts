import { describe, it, expect } from 'vitest';
import {
  demoBfs,
  demoDfs,
  demoPagerank,
  demoComponents,
  demoBetweenness,
  demoCloseness,
  demoEigenvector,
  demoLouvain,
  demoInfomap,
  demoSpinglass,
  demoLabelPropagation,
  demoWalktrap,
  demoLeiden,
  demoFastGreedy,
  demoLeadingEigenvector,
  demoEdgeBetweennessCommunity,
  demoFluid,
  demoHarmonic,
  demoHits,
  demoKatz,
  demoTriadCensus,
  demoCanonicalPermutation,
  demoCountAutomorphisms,
  demoIsomorphism,
  demoCoreness,
  demoEccentricity,
  demoConstraint,
  demoDiameter,
  demoShortestPath,
  demoRandomWalk,
  runDemoAlgo,
  layoutFR,
} from './algorithms';
import type {
  Edge, AlgoResultScores, AlgoResultMembership, AlgoResultOrder, AlgoResultHits,
  AlgoResultTriadCensus, AlgoResultPermutation, AlgoResultAutomorphisms, AlgoResultIsomorphism,
  AlgoResultCores, AlgoResultValues, AlgoResultPath, AlgoResultWalk, AlgoResultDiameter,
} from './types';

const TRIANGLE: Edge[] = [[0, 1], [1, 2], [2, 0]];
const PATH: Edge[] = [[0, 1], [1, 2], [2, 3]];
const DISCONNECTED: Edge[] = [[0, 1], [2, 3]];

describe('demoBfs', () => {
  it('visits all vertices in a connected graph', () => {
    const result = demoBfs(3, TRIANGLE) as AlgoResultOrder;
    expect(result.order).toHaveLength(3);
    expect(new Set(result.order)).toEqual(new Set([0, 1, 2]));
  });

  it('starts from the default source vertex 0', () => {
    const result = demoBfs(4, PATH) as AlgoResultOrder;
    expect(result.order[0]).toBe(0);
  });

  it('respects the source parameter', () => {
    const result = demoBfs(4, PATH, 3) as AlgoResultOrder;
    expect(result.order[0]).toBe(3);
  });

  it('clamps out-of-range source to 0', () => {
    const result = demoBfs(4, PATH, 99) as AlgoResultOrder;
    expect(result.order[0]).toBe(0);
  });

  it('clamps negative source to 0', () => {
    const result = demoBfs(4, PATH, -1) as AlgoResultOrder;
    expect(result.order[0]).toBe(0);
  });

  it('only visits reachable vertices from disconnected graph', () => {
    const result = demoBfs(4, DISCONNECTED, 0) as AlgoResultOrder;
    expect(result.order).toEqual([0, 1]);
  });

  it('handles single vertex graph', () => {
    const result = demoBfs(1, []) as AlgoResultOrder;
    expect(result.order).toEqual([0]);
  });
});

describe('demoPagerank', () => {
  it('returns scores summing to approximately 1', () => {
    const result = demoPagerank(3, TRIANGLE) as AlgoResultScores;
    const sum = result.scores.reduce((a, b) => a + b, 0);
    expect(sum).toBeCloseTo(1.0, 4);
  });

  it('returns equal scores for a symmetric graph', () => {
    const result = demoPagerank(3, TRIANGLE) as AlgoResultScores;
    expect(result.scores[0]).toBeCloseTo(result.scores[1]!, 6);
    expect(result.scores[1]).toBeCloseTo(result.scores[2]!, 6);
  });

  it('returns one score per vertex', () => {
    const result = demoPagerank(4, PATH) as AlgoResultScores;
    expect(result.scores).toHaveLength(4);
  });

  it('accepts custom damping factor', () => {
    const r1 = demoPagerank(4, PATH, 0.5) as AlgoResultScores;
    const r2 = demoPagerank(4, PATH, 0.99) as AlgoResultScores;
    expect(r1.scores[0]).not.toBeCloseTo(r2.scores[0]!, 4);
  });

  it('all scores are non-negative', () => {
    const result = demoPagerank(4, PATH) as AlgoResultScores;
    for (const s of result.scores) {
      expect(s).toBeGreaterThanOrEqual(0);
    }
  });
});

describe('demoComponents', () => {
  it('finds one component in a connected graph', () => {
    const result = demoComponents(3, TRIANGLE) as AlgoResultMembership;
    expect(result.count).toBe(1);
    expect(new Set(result.membership).size).toBe(1);
  });

  it('finds two components in a disconnected graph', () => {
    const result = demoComponents(4, DISCONNECTED) as AlgoResultMembership;
    expect(result.count).toBe(2);
    expect(result.membership[0]).toBe(result.membership[1]);
    expect(result.membership[2]).toBe(result.membership[3]);
    expect(result.membership[0]).not.toBe(result.membership[2]);
  });

  it('assigns one component per isolated vertex', () => {
    const result = demoComponents(3, []) as AlgoResultMembership;
    expect(result.count).toBe(3);
  });

  it('membership length equals vcount', () => {
    const result = demoComponents(4, PATH) as AlgoResultMembership;
    expect(result.membership).toHaveLength(4);
  });
});

describe('demoBetweenness', () => {
  it('returns one score per vertex', () => {
    const result = demoBetweenness(4, PATH) as AlgoResultScores;
    expect(result.scores).toHaveLength(4);
  });

  it('endpoint vertices have zero betweenness on a path', () => {
    const result = demoBetweenness(4, PATH) as AlgoResultScores;
    expect(result.scores[0]).toBe(0);
    expect(result.scores[3]).toBe(0);
  });

  it('internal vertices have higher betweenness on a path', () => {
    const result = demoBetweenness(4, PATH) as AlgoResultScores;
    expect(result.scores[1]).toBeGreaterThan(0);
    expect(result.scores[2]).toBeGreaterThan(0);
  });

  it('all scores are non-negative', () => {
    const result = demoBetweenness(4, PATH) as AlgoResultScores;
    for (const s of result.scores) {
      expect(s).toBeGreaterThanOrEqual(0);
    }
  });
});

describe('demoLouvain', () => {
  it('returns membership array', () => {
    const result = demoLouvain(3, TRIANGLE) as AlgoResultMembership;
    expect(result.membership).toHaveLength(3);
    expect(result.modularity).toBeDefined();
  });
});

describe('demoInfomap', () => {
  it('returns membership and codelength', () => {
    const result = demoInfomap(3, TRIANGLE) as AlgoResultMembership;
    expect(result.membership).toHaveLength(3);
    expect(result.codelength).toBeDefined();
  });
});

describe('demoSpinglass', () => {
  it('returns membership, modularity, and nb_clusters', () => {
    const result = demoSpinglass(3, TRIANGLE) as AlgoResultMembership;
    expect(result.membership).toHaveLength(3);
    expect(result.modularity).toBeDefined();
    expect(result.nb_clusters).toBeDefined();
  });
});

describe('demoCloseness', () => {
  it('returns one score per vertex', () => {
    const result = demoCloseness(4, PATH) as AlgoResultScores;
    expect(result.scores).toHaveLength(4);
  });

  it('all scores are non-negative', () => {
    const result = demoCloseness(3, TRIANGLE) as AlgoResultScores;
    for (const s of result.scores) {
      expect(s).toBeGreaterThanOrEqual(0);
    }
  });
});

describe('demoEigenvector', () => {
  it('returns scores summing to approximately 1', () => {
    const result = demoEigenvector(3, TRIANGLE) as AlgoResultScores;
    const sum = result.scores.reduce((a, b) => a + b, 0);
    expect(sum).toBeCloseTo(1.0, 4);
  });

  it('returns one score per vertex', () => {
    const result = demoEigenvector(4, PATH) as AlgoResultScores;
    expect(result.scores).toHaveLength(4);
  });
});

describe('demoDfs', () => {
  it('visits all vertices in a connected graph', () => {
    const result = demoDfs(3, TRIANGLE) as AlgoResultOrder;
    expect(result.order).toHaveLength(3);
    expect(new Set(result.order)).toEqual(new Set([0, 1, 2]));
  });

  it('starts from the default source vertex 0', () => {
    const result = demoDfs(4, PATH) as AlgoResultOrder;
    expect(result.order[0]).toBe(0);
  });

  it('respects the source parameter', () => {
    const result = demoDfs(4, PATH, 3) as AlgoResultOrder;
    expect(result.order[0]).toBe(3);
  });

  it('only visits reachable vertices from disconnected graph', () => {
    const result = demoDfs(4, DISCONNECTED, 0) as AlgoResultOrder;
    expect(result.order).toEqual([0, 1]);
  });

  it('handles empty graph', () => {
    const result = demoDfs(0, []) as AlgoResultOrder;
    expect(result.order).toEqual([]);
  });
});

describe('demoLabelPropagation', () => {
  it('returns membership and nb_clusters', () => {
    const result = demoLabelPropagation(3, TRIANGLE) as AlgoResultMembership;
    expect(result.membership).toHaveLength(3);
    expect(result.nb_clusters).toBeDefined();
  });
});

describe('demoWalktrap', () => {
  it('returns membership and modularity', () => {
    const result = demoWalktrap(3, TRIANGLE) as AlgoResultMembership;
    expect(result.membership).toHaveLength(3);
    expect(result.modularity).toBeDefined();
    expect(result.nb_clusters).toBeDefined();
  });
});

describe('demoLeiden', () => {
  it('returns membership and quality', () => {
    const result = demoLeiden(3, TRIANGLE) as AlgoResultMembership;
    expect(result.membership).toHaveLength(3);
    expect(result.quality).toBeDefined();
    expect(result.nb_clusters).toBeDefined();
  });
});

describe('demoFastGreedy', () => {
  it('returns membership and modularity', () => {
    const result = demoFastGreedy(3, TRIANGLE) as AlgoResultMembership;
    expect(result.membership).toHaveLength(3);
    expect(result.modularity).toBeDefined();
    expect(result.nb_clusters).toBeDefined();
  });
});

describe('demoLeadingEigenvector', () => {
  it('returns membership and modularity', () => {
    const result = demoLeadingEigenvector(3, TRIANGLE) as AlgoResultMembership;
    expect(result.membership).toHaveLength(3);
    expect(result.modularity).toBeDefined();
  });
});

describe('demoEdgeBetweennessCommunity', () => {
  it('returns membership and nb_clusters', () => {
    const result = demoEdgeBetweennessCommunity(3, TRIANGLE) as AlgoResultMembership;
    expect(result.membership).toHaveLength(3);
    expect(result.nb_clusters).toBeDefined();
  });
});

describe('demoFluid', () => {
  it('returns membership and nb_clusters', () => {
    const result = demoFluid(3, TRIANGLE) as AlgoResultMembership;
    expect(result.membership).toHaveLength(3);
    expect(result.nb_clusters).toBeDefined();
  });
});

describe('demoHarmonic', () => {
  it('returns one score per vertex', () => {
    const result = demoHarmonic(4, PATH) as AlgoResultScores;
    expect(result.scores).toHaveLength(4);
  });

  it('all scores are non-negative', () => {
    const result = demoHarmonic(3, TRIANGLE) as AlgoResultScores;
    for (const s of result.scores) {
      expect(s).toBeGreaterThanOrEqual(0);
    }
  });

  it('central vertices have higher harmonic centrality on a path', () => {
    const result = demoHarmonic(4, PATH) as AlgoResultScores;
    expect(result.scores[1]!).toBeGreaterThan(result.scores[0]!);
    expect(result.scores[2]!).toBeGreaterThan(result.scores[3]!);
  });
});

describe('demoHits', () => {
  it('returns hub and authority arrays', () => {
    const result = demoHits(3, TRIANGLE) as AlgoResultHits;
    expect(result.hub).toHaveLength(3);
    expect(result.authority).toHaveLength(3);
  });

  it('scores are non-negative', () => {
    const result = demoHits(4, PATH) as AlgoResultHits;
    for (const h of result.hub) expect(h).toBeGreaterThanOrEqual(0);
    for (const a of result.authority) expect(a).toBeGreaterThanOrEqual(0);
  });

  it('hub and authority are normalized', () => {
    const result = demoHits(3, TRIANGLE) as AlgoResultHits;
    const hubNorm = Math.sqrt(result.hub.reduce((s, x) => s + x * x, 0));
    const authNorm = Math.sqrt(result.authority.reduce((s, x) => s + x * x, 0));
    expect(hubNorm).toBeCloseTo(1.0, 4);
    expect(authNorm).toBeCloseTo(1.0, 4);
  });
});

describe('demoKatz', () => {
  it('returns one score per vertex', () => {
    const result = demoKatz(4, PATH) as AlgoResultScores;
    expect(result.scores).toHaveLength(4);
  });

  it('all scores are positive', () => {
    const result = demoKatz(3, TRIANGLE) as AlgoResultScores;
    for (const s of result.scores) {
      expect(s).toBeGreaterThan(0);
    }
  });
});

describe('demoTriadCensus', () => {
  it('returns 16 counts', () => {
    const result = demoTriadCensus(5, PATH) as AlgoResultTriadCensus;
    expect(result.counts).toHaveLength(16);
  });

  it('all counts are non-negative', () => {
    const result = demoTriadCensus(4, TRIANGLE) as AlgoResultTriadCensus;
    for (const c of result.counts) {
      expect(c).toBeGreaterThanOrEqual(0);
    }
  });

  it('total triples include edge count', () => {
    const result = demoTriadCensus(4, PATH) as AlgoResultTriadCensus;
    expect(result.counts[1]).toBe(3);
  });
});

describe('demoCanonicalPermutation', () => {
  it('returns identity permutation of correct length', () => {
    const result = demoCanonicalPermutation(4, PATH) as AlgoResultPermutation;
    expect(result.permutation).toHaveLength(4);
    expect(result.permutation).toEqual([0, 1, 2, 3]);
  });

  it('handles empty graph', () => {
    const result = demoCanonicalPermutation(0, []) as AlgoResultPermutation;
    expect(result.permutation).toHaveLength(0);
  });
});

describe('demoCountAutomorphisms', () => {
  it('returns a positive count', () => {
    const result = demoCountAutomorphisms(4, PATH) as AlgoResultAutomorphisms;
    expect(result.count).toBeGreaterThan(0);
  });
});

describe('demoIsomorphism', () => {
  it('returns isomorphic true with identity mapping', () => {
    const result = demoIsomorphism(3, TRIANGLE) as AlgoResultIsomorphism;
    expect(result.isomorphic).toBe(true);
    expect(result.mapping).toHaveLength(3);
    expect(result.mapping).toEqual([0, 1, 2]);
  });
});

describe('demoCoreness', () => {
  it('returns one core value per vertex', () => {
    const result = demoCoreness(4, PATH) as AlgoResultCores;
    expect(result.cores).toHaveLength(4);
  });

  it('all core values are non-negative', () => {
    const result = demoCoreness(3, TRIANGLE) as AlgoResultCores;
    for (const c of result.cores) {
      expect(c).toBeGreaterThanOrEqual(0);
    }
  });

  it('triangle vertices have core value 2', () => {
    const result = demoCoreness(3, TRIANGLE) as AlgoResultCores;
    for (const c of result.cores) {
      expect(c).toBe(2);
    }
  });

  it('path endpoints have core value 1', () => {
    const result = demoCoreness(4, PATH) as AlgoResultCores;
    expect(result.cores[0]).toBe(1);
    expect(result.cores[3]).toBe(1);
  });

  it('handles isolated vertices', () => {
    const result = demoCoreness(3, []) as AlgoResultCores;
    expect(result.cores).toEqual([0, 0, 0]);
  });
});

describe('demoEccentricity', () => {
  it('returns one value per vertex', () => {
    const result = demoEccentricity(4, PATH) as AlgoResultValues;
    expect(result.values).toHaveLength(4);
  });

  it('all values are non-negative', () => {
    const result = demoEccentricity(3, TRIANGLE) as AlgoResultValues;
    for (const v of result.values) {
      expect(v).toBeGreaterThanOrEqual(0);
    }
  });

  it('path endpoints have highest eccentricity', () => {
    const result = demoEccentricity(4, PATH) as AlgoResultValues;
    expect(result.values[0]).toBe(3);
    expect(result.values[3]).toBe(3);
    expect(result.values[1]).toBe(2);
    expect(result.values[2]).toBe(2);
  });

  it('triangle vertices all have eccentricity 1', () => {
    const result = demoEccentricity(3, TRIANGLE) as AlgoResultValues;
    expect(result.values).toEqual([1, 1, 1]);
  });
});

describe('demoConstraint', () => {
  it('returns one score per vertex', () => {
    const result = demoConstraint(4, PATH) as AlgoResultScores;
    expect(result.scores).toHaveLength(4);
  });

  it('all scores are non-negative', () => {
    const result = demoConstraint(3, TRIANGLE) as AlgoResultScores;
    for (const s of result.scores) {
      expect(s).toBeGreaterThanOrEqual(0);
    }
  });

  it('isolated vertices have zero constraint', () => {
    const result = demoConstraint(3, []) as AlgoResultScores;
    expect(result.scores).toEqual([0, 0, 0]);
  });
});

describe('demoDiameter', () => {
  it('returns diameter for a path graph', () => {
    const result = demoDiameter(4, PATH) as AlgoResultDiameter;
    expect(result.diameter).toBe(3);
  });

  it('returns diameter 1 for a triangle', () => {
    const result = demoDiameter(3, TRIANGLE) as AlgoResultDiameter;
    expect(result.diameter).toBe(1);
  });

  it('returns null for empty graph', () => {
    const result = demoDiameter(0, []) as AlgoResultDiameter;
    expect(result.diameter).toBeNull();
  });

  it('returns 0 for single vertex', () => {
    const result = demoDiameter(1, []) as AlgoResultDiameter;
    expect(result.diameter).toBe(0);
  });
});

describe('demoShortestPath', () => {
  it('finds path between connected vertices', () => {
    const result = demoShortestPath(4, PATH, 0, 3) as AlgoResultPath;
    expect(result.path).toEqual([0, 1, 2, 3]);
  });

  it('finds direct neighbor path', () => {
    const result = demoShortestPath(3, TRIANGLE, 0, 1) as AlgoResultPath;
    expect(result.path).toEqual([0, 1]);
  });

  it('returns empty path for unreachable target', () => {
    const result = demoShortestPath(4, DISCONNECTED, 0, 2) as AlgoResultPath;
    expect(result.path).toEqual([]);
  });

  it('handles empty graph', () => {
    const result = demoShortestPath(0, []) as AlgoResultPath;
    expect(result.path).toEqual([]);
  });

  it('uses default source and target', () => {
    const result = demoShortestPath(4, PATH) as AlgoResultPath;
    expect(result.path[0]).toBe(0);
    expect(result.path[result.path.length - 1]).toBe(1);
  });
});

describe('demoRandomWalk', () => {
  it('starts from specified source', () => {
    const result = demoRandomWalk(4, PATH, 2) as AlgoResultWalk;
    expect(result.vertices[0]).toBe(2);
  });

  it('returns non-empty walk on connected graph', () => {
    const result = demoRandomWalk(3, TRIANGLE, 0) as AlgoResultWalk;
    expect(result.vertices.length).toBeGreaterThan(0);
  });

  it('all visited vertices are valid', () => {
    const result = demoRandomWalk(4, PATH, 0) as AlgoResultWalk;
    for (const v of result.vertices) {
      expect(v).toBeGreaterThanOrEqual(0);
      expect(v).toBeLessThan(4);
    }
  });

  it('handles empty graph', () => {
    const result = demoRandomWalk(0, []) as AlgoResultWalk;
    expect(result.vertices).toEqual([]);
  });

  it('stops early on isolated vertex', () => {
    const result = demoRandomWalk(3, [], 0) as AlgoResultWalk;
    expect(result.vertices).toEqual([0]);
  });
});

describe('runDemoAlgo', () => {
  it('dispatches to bfs', () => {
    const result = runDemoAlgo('bfs', 4, PATH, { source: 2 }) as AlgoResultOrder;
    expect(result.order[0]).toBe(2);
  });

  it('dispatches to pagerank', () => {
    const result = runDemoAlgo('pagerank', 3, TRIANGLE) as AlgoResultScores;
    expect(result.scores).toHaveLength(3);
  });

  it('dispatches to components', () => {
    const result = runDemoAlgo('components', 4, DISCONNECTED) as AlgoResultMembership;
    expect(result.count).toBe(2);
  });

  it('dispatches to betweenness', () => {
    const result = runDemoAlgo('betweenness', 4, PATH) as AlgoResultScores;
    expect(result.scores).toHaveLength(4);
  });

  it('dispatches to louvain', () => {
    const result = runDemoAlgo('louvain', 3, TRIANGLE) as AlgoResultMembership;
    expect(result.membership).toBeDefined();
  });

  it('dispatches to infomap', () => {
    const result = runDemoAlgo('infomap', 3, TRIANGLE) as AlgoResultMembership;
    expect(result.codelength).toBeDefined();
  });

  it('dispatches to spinglass', () => {
    const result = runDemoAlgo('spinglass', 3, TRIANGLE) as AlgoResultMembership;
    expect(result.nb_clusters).toBeDefined();
  });

  it('dispatches to closeness', () => {
    const result = runDemoAlgo('closeness', 4, PATH) as AlgoResultScores;
    expect(result.scores).toHaveLength(4);
  });

  it('dispatches to eigenvector', () => {
    const result = runDemoAlgo('eigenvector', 3, TRIANGLE) as AlgoResultScores;
    expect(result.scores).toHaveLength(3);
  });

  it('dispatches to dfs', () => {
    const result = runDemoAlgo('dfs', 4, PATH, { source: 2 }) as AlgoResultOrder;
    expect(result.order[0]).toBe(2);
  });

  it('dispatches to label_propagation', () => {
    const result = runDemoAlgo('label_propagation', 3, TRIANGLE) as AlgoResultMembership;
    expect(result.nb_clusters).toBeDefined();
  });

  it('dispatches to walktrap', () => {
    const result = runDemoAlgo('walktrap', 3, TRIANGLE) as AlgoResultMembership;
    expect(result.modularity).toBeDefined();
  });

  it('dispatches to leiden', () => {
    const result = runDemoAlgo('leiden', 3, TRIANGLE) as AlgoResultMembership;
    expect(result.quality).toBeDefined();
  });

  it('dispatches to fast_greedy', () => {
    const result = runDemoAlgo('fast_greedy', 3, TRIANGLE) as AlgoResultMembership;
    expect(result.modularity).toBeDefined();
  });

  it('dispatches to leading_eigenvector', () => {
    const result = runDemoAlgo('leading_eigenvector', 3, TRIANGLE) as AlgoResultMembership;
    expect(result.modularity).toBeDefined();
  });

  it('dispatches to edge_betweenness', () => {
    const result = runDemoAlgo('edge_betweenness', 3, TRIANGLE) as AlgoResultMembership;
    expect(result.nb_clusters).toBeDefined();
  });

  it('dispatches to fluid', () => {
    const result = runDemoAlgo('fluid', 3, TRIANGLE) as AlgoResultMembership;
    expect(result.nb_clusters).toBeDefined();
  });

  it('dispatches to harmonic', () => {
    const result = runDemoAlgo('harmonic', 4, PATH) as AlgoResultScores;
    expect(result.scores).toHaveLength(4);
  });

  it('dispatches to hits', () => {
    const result = runDemoAlgo('hits', 3, TRIANGLE) as AlgoResultHits;
    expect(result.hub).toHaveLength(3);
    expect(result.authority).toHaveLength(3);
  });

  it('dispatches to katz', () => {
    const result = runDemoAlgo('katz', 3, TRIANGLE) as AlgoResultScores;
    expect(result.scores).toHaveLength(3);
  });

  it('dispatches to triad_census', () => {
    const result = runDemoAlgo('triad_census', 4, PATH) as AlgoResultTriadCensus;
    expect(result.counts).toHaveLength(16);
  });

  it('dispatches to canonical_permutation', () => {
    const result = runDemoAlgo('canonical_permutation', 4, PATH) as AlgoResultPermutation;
    expect(result.permutation).toHaveLength(4);
  });

  it('dispatches to count_automorphisms', () => {
    const result = runDemoAlgo('count_automorphisms', 4, PATH) as AlgoResultAutomorphisms;
    expect(result.count).toBeGreaterThan(0);
  });

  it('dispatches to isomorphism', () => {
    const result = runDemoAlgo('isomorphism', 3, TRIANGLE) as AlgoResultIsomorphism;
    expect(result.isomorphic).toBe(true);
  });

  it('dispatches to coreness', () => {
    const result = runDemoAlgo('coreness', 3, TRIANGLE) as AlgoResultCores;
    expect(result.cores).toHaveLength(3);
  });

  it('dispatches to eccentricity', () => {
    const result = runDemoAlgo('eccentricity', 4, PATH) as AlgoResultValues;
    expect(result.values).toHaveLength(4);
  });

  it('dispatches to constraint', () => {
    const result = runDemoAlgo('constraint', 3, TRIANGLE) as AlgoResultScores;
    expect(result.scores).toHaveLength(3);
  });

  it('dispatches to diameter', () => {
    const result = runDemoAlgo('diameter', 4, PATH) as AlgoResultDiameter;
    expect(result.diameter).toBe(3);
  });

  it('dispatches to shortest_path', () => {
    const result = runDemoAlgo('shortest_path', 4, PATH, { source: 0, target: 3 }) as AlgoResultPath;
    expect(result.path).toEqual([0, 1, 2, 3]);
  });

  it('dispatches to random_walk', () => {
    const result = runDemoAlgo('random_walk', 3, TRIANGLE, { source: 0 }) as AlgoResultWalk;
    expect(result.vertices[0]).toBe(0);
    expect(result.vertices.length).toBeGreaterThan(0);
  });

  it('falls back to pagerank for unknown algo', () => {
    const result = runDemoAlgo('unknown', 3, TRIANGLE) as AlgoResultScores;
    expect(result.scores).toHaveLength(3);
  });
});

describe('layoutFR', () => {
  it('returns one coordinate pair per vertex', () => {
    const coords = layoutFR(3, TRIANGLE, 10);
    expect(coords).toHaveLength(3);
    for (const [x, y] of coords) {
      expect(x).toBeGreaterThanOrEqual(0);
      expect(x).toBeLessThanOrEqual(1);
      expect(y).toBeGreaterThanOrEqual(0);
      expect(y).toBeLessThanOrEqual(1);
    }
  });

  it('handles zero vertices', () => {
    const coords = layoutFR(0, [], 10);
    expect(coords).toHaveLength(0);
  });

  it('handles isolated vertices', () => {
    const coords = layoutFR(3, [], 10);
    expect(coords).toHaveLength(3);
  });
});
