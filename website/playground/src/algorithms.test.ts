import { describe, it, expect } from 'vitest';
import {
  demoBfs,
  demoPagerank,
  demoComponents,
  demoBetweenness,
  demoLouvain,
  demoInfomap,
  demoSpinglass,
  runDemoAlgo,
  layoutFR,
} from './algorithms';
import type { Edge, AlgoResultScores, AlgoResultMembership, AlgoResultOrder } from './types';

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
