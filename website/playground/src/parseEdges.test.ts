import { describe, it, expect } from 'vitest';

function parseEdges(text: string): [number, number][] {
  const edges: [number, number][] = [];
  for (const line of text.split('\n')) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#') || trimmed.startsWith('//')) continue;
    const parts = trimmed.split(/[\s,;]+/).map(Number);
    if (parts.length >= 2 && Number.isFinite(parts[0]) && Number.isFinite(parts[1])) {
      edges.push([parts[0]!, parts[1]!]);
    }
  }
  return edges;
}

function getVcount(edges: [number, number][]): number {
  let max = -1;
  for (const [u, v] of edges) {
    if (u > max) max = u;
    if (v > max) max = v;
  }
  return max + 1;
}

describe('parseEdges', () => {
  it('parses space-separated edges', () => {
    expect(parseEdges('0 1\n1 2')).toEqual([[0, 1], [1, 2]]);
  });

  it('parses comma-separated edges', () => {
    expect(parseEdges('0,1\n1,2')).toEqual([[0, 1], [1, 2]]);
  });

  it('parses semicolon-separated edges', () => {
    expect(parseEdges('0;1\n1;2')).toEqual([[0, 1], [1, 2]]);
  });

  it('parses tab-separated edges', () => {
    expect(parseEdges('0\t1\n1\t2')).toEqual([[0, 1], [1, 2]]);
  });

  it('skips comment lines starting with #', () => {
    expect(parseEdges('# comment\n0 1')).toEqual([[0, 1]]);
  });

  it('skips comment lines starting with //', () => {
    expect(parseEdges('// comment\n0 1')).toEqual([[0, 1]]);
  });

  it('skips empty lines', () => {
    expect(parseEdges('\n0 1\n\n1 2\n')).toEqual([[0, 1], [1, 2]]);
  });

  it('handles whitespace-only lines', () => {
    expect(parseEdges('   \n0 1')).toEqual([[0, 1]]);
  });

  it('ignores lines with non-numeric content', () => {
    expect(parseEdges('abc def\n0 1')).toEqual([[0, 1]]);
  });

  it('ignores lines with only one number', () => {
    expect(parseEdges('42\n0 1')).toEqual([[0, 1]]);
  });

  it('returns empty array for empty input', () => {
    expect(parseEdges('')).toEqual([]);
  });

  it('handles mixed separators', () => {
    expect(parseEdges('0, 1\n1; 2')).toEqual([[0, 1], [1, 2]]);
  });

  it('handles extra whitespace around numbers', () => {
    expect(parseEdges('  0   1  ')).toEqual([[0, 1]]);
  });
});

describe('getVcount', () => {
  it('returns 0 for empty edge list', () => {
    expect(getVcount([])).toBe(0);
  });

  it('returns max vertex + 1', () => {
    expect(getVcount([[0, 1], [2, 5]])).toBe(6);
  });

  it('handles single edge', () => {
    expect(getVcount([[0, 1]])).toBe(2);
  });

  it('handles vertex 0 only', () => {
    expect(getVcount([[0, 0]])).toBe(1);
  });
});
