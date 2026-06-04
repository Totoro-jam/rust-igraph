import { describe, it, expect } from 'vitest';
import type { GeneratorId, GeneratorParams, GeneratedGraph } from './types';

describe('Generator types', () => {
  const ALL_GENERATORS: GeneratorId[] = [
    'erdos_renyi', 'barabasi_albert', 'watts_strogatz',
    'complete', 'cycle', 'path', 'star', 'ring', 'famous',
  ];

  it('defines 9 generator models', () => {
    expect(ALL_GENERATORS).toHaveLength(9);
  });

  it('GeneratorParams accepts valid configurations', () => {
    const erdos: GeneratorParams = { n: 50, p: 0.1, seed: 42 };
    expect(erdos.n).toBe(50);
    expect(erdos.p).toBe(0.1);

    const ba: GeneratorParams = { n: 100, m: 3, seed: 7 };
    expect(ba.m).toBe(3);

    const ws: GeneratorParams = { n: 30, k: 4, p: 0.1, seed: 1 };
    expect(ws.k).toBe(4);

    const famous: GeneratorParams = { name: 'Petersen' };
    expect(famous.name).toBe('Petersen');

    const path: GeneratorParams = { n: 10, directed: true };
    expect(path.directed).toBe(true);
  });

  it('GeneratedGraph has expected shape', () => {
    const g: GeneratedGraph = {
      edges: [[0, 1], [1, 2], [2, 0]],
      directed: false,
      vcount: 3,
    };
    expect(g.edges).toHaveLength(3);
    expect(g.vcount).toBe(3);
    expect(g.directed).toBe(false);
  });

  it('each generator id is unique', () => {
    const unique = new Set(ALL_GENERATORS);
    expect(unique.size).toBe(ALL_GENERATORS.length);
  });
});
