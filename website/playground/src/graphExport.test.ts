import { describe, it, expect, vi, beforeEach } from 'vitest';
import { exportGml, exportDot, exportGraphml, exportEdgeList } from './graphExport';
import type { Edge } from './types';

const TRIANGLE: Edge[] = [[0, 1], [1, 2], [2, 0]];

let capturedBlob: Blob | null = null;
let capturedFilename: string | null = null;

beforeEach(() => {
  capturedBlob = null;
  capturedFilename = null;

  vi.stubGlobal('URL', {
    createObjectURL: vi.fn(() => 'blob:mock'),
    revokeObjectURL: vi.fn(),
  });

  vi.spyOn(document, 'createElement').mockImplementation((tag: string) => {
    if (tag === 'a') {
      const obj: Record<string, unknown> = {
        download: '',
        href: '',
        click: vi.fn(),
      };
      Object.defineProperty(obj, 'download', {
        get() { return capturedFilename; },
        set(v: string) { capturedFilename = v; },
      });
      return obj as unknown as HTMLElement;
    }
    return document.createElement(tag);
  });

  const origBlob = globalThis.Blob;
  vi.stubGlobal('Blob', class MockBlob extends origBlob {
    constructor(parts?: BlobPart[], options?: BlobPropertyBag) {
      super(parts, options);
      capturedBlob = this;
    }
  });
});

async function getCapturedText(): Promise<string> {
  if (!capturedBlob) throw new Error('No blob captured');
  return capturedBlob.text();
}

describe('exportGml', () => {
  it('generates valid GML for undirected graph', async () => {
    exportGml(TRIANGLE, 3, false);
    expect(capturedFilename).toBe('graph.gml');
    const text = await getCapturedText();
    expect(text).toContain('directed 0');
    expect(text).toContain('node [');
    expect(text).toContain('id 0');
    expect(text).toContain('id 1');
    expect(text).toContain('id 2');
    expect(text).toContain('edge [');
    expect(text).toContain('source 0');
    expect(text).toContain('target 1');
  });

  it('sets directed 1 for directed graphs', async () => {
    exportGml(TRIANGLE, 3, true);
    const text = await getCapturedText();
    expect(text).toContain('directed 1');
  });
});

describe('exportDot', () => {
  it('generates undirected DOT', async () => {
    exportDot(TRIANGLE, 3, false);
    expect(capturedFilename).toBe('graph.dot');
    const text = await getCapturedText();
    expect(text).toContain('graph G');
    expect(text).toContain('0 -- 1');
    expect(text).not.toContain('->');
  });

  it('generates directed DOT', async () => {
    exportDot(TRIANGLE, 3, true);
    const text = await getCapturedText();
    expect(text).toContain('digraph G');
    expect(text).toContain('0 -> 1');
  });
});

describe('exportGraphml', () => {
  it('generates valid GraphML XML', async () => {
    exportGraphml(TRIANGLE, 3, false);
    expect(capturedFilename).toBe('graph.graphml');
    const text = await getCapturedText();
    expect(text).toContain('<?xml');
    expect(text).toContain('<graphml');
    expect(text).toContain('edgedefault="undirected"');
    expect(text).toContain('<node id="n0"');
    expect(text).toContain('<edge id="e0" source="n0" target="n1"');
  });

  it('sets directed for directed graphs', async () => {
    exportGraphml(TRIANGLE, 3, true);
    const text = await getCapturedText();
    expect(text).toContain('edgedefault="directed"');
  });
});

describe('exportEdgeList', () => {
  it('generates edge list text', async () => {
    exportEdgeList(TRIANGLE, 3, false);
    expect(capturedFilename).toBe('graph.txt');
    const text = await getCapturedText();
    const lines = text.trim().split('\n');
    expect(lines).toHaveLength(3);
    expect(lines[0]).toBe('0 1');
    expect(lines[1]).toBe('1 2');
    expect(lines[2]).toBe('2 0');
  });
});
