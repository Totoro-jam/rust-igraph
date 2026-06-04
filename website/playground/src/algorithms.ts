import type { Edge, AlgoResult } from './types';

export function layoutFR(
  vcount: number,
  edges: Edge[],
  niter: number,
): [number, number][] {
  const k = Math.sqrt(1 / Math.max(vcount, 1));
  const coords: [number, number][] = Array.from({ length: vcount }, () => [
    Math.random(),
    Math.random(),
  ]);

  for (let iter = 0; iter < niter; iter++) {
    const temp = (1 - iter / niter) * 0.1;
    const disp: [number, number][] = coords.map(() => [0, 0]);

    for (let i = 0; i < vcount; i++) {
      for (let j = i + 1; j < vcount; j++) {
        let dx = coords[i]![0] - coords[j]![0];
        let dy = coords[i]![1] - coords[j]![1];
        const dist = Math.sqrt(dx * dx + dy * dy) || 0.001;
        const force = (k * k) / dist;
        const fx = (dx / dist) * force;
        const fy = (dy / dist) * force;
        disp[i]![0] += fx;
        disp[i]![1] += fy;
        disp[j]![0] -= fx;
        disp[j]![1] -= fy;
      }
    }

    for (const [u, v] of edges) {
      if (u >= vcount || v >= vcount) continue;
      const dx = coords[u]![0] - coords[v]![0];
      const dy = coords[u]![1] - coords[v]![1];
      const dist = Math.sqrt(dx * dx + dy * dy) || 0.001;
      const force = (dist * dist) / k;
      const fx = (dx / dist) * force;
      const fy = (dy / dist) * force;
      disp[u]![0] -= fx;
      disp[u]![1] -= fy;
      disp[v]![0] += fx;
      disp[v]![1] += fy;
    }

    for (let i = 0; i < vcount; i++) {
      const dx = disp[i]![0],
        dy = disp[i]![1];
      const dist = Math.sqrt(dx * dx + dy * dy) || 0.001;
      const cap = Math.min(dist, temp);
      coords[i]![0] += (dx / dist) * cap;
      coords[i]![1] += (dy / dist) * cap;
      coords[i]![0] = Math.max(0.05, Math.min(0.95, coords[i]![0]));
      coords[i]![1] = Math.max(0.05, Math.min(0.95, coords[i]![1]));
    }
  }
  return coords;
}

function buildAdj(vcount: number, edges: Edge[]): number[][] {
  const adj: number[][] = Array.from({ length: vcount }, () => []);
  for (const [u, v] of edges) {
    if (u < vcount && v < vcount) {
      adj[u]!.push(v);
      adj[v]!.push(u);
    }
  }
  return adj;
}

export function demoBfs(vcount: number, edges: Edge[], source = 0): AlgoResult {
  const adj = buildAdj(vcount, edges);
  const visited = new Set<number>();
  const order: number[] = [];
  const start = source >= 0 && source < vcount ? source : 0;
  const queue = [start];
  visited.add(start);
  while (queue.length > 0) {
    const v = queue.shift()!;
    order.push(v);
    for (const w of adj[v]!) {
      if (!visited.has(w)) {
        visited.add(w);
        queue.push(w);
      }
    }
  }
  return { order };
}

export function demoPagerank(
  vcount: number,
  edges: Edge[],
  damping = 0.85,
): AlgoResult {
  const scores = new Float64Array(vcount).fill(1 / vcount);
  const outDeg = new Float64Array(vcount);
  const inAdj: number[][] = Array.from({ length: vcount }, () => []);
  for (const [u, v] of edges) {
    if (u < vcount && v < vcount) {
      inAdj[v]!.push(u);
      outDeg[u]!++;
    }
  }
  for (let t = 0; t < 100; t++) {
    const next = new Float64Array(vcount).fill((1 - damping) / vcount);
    for (let v = 0; v < vcount; v++) {
      for (const u of inAdj[v]!) {
        if (outDeg[u]! > 0) next[v]! += (damping * scores[u]!) / outDeg[u]!;
      }
    }
    scores.set(next);
  }
  return { scores: Array.from(scores) };
}

export function demoComponents(vcount: number, edges: Edge[]): AlgoResult {
  const parent = Array.from({ length: vcount }, (_, i) => i);
  function find(x: number): number {
    while (parent[x] !== x) {
      parent[x] = parent[parent[x]!]!;
      x = parent[x]!;
    }
    return x;
  }
  function union(a: number, b: number) {
    parent[find(a)] = find(b);
  }
  for (const [u, v] of edges) {
    if (u < vcount && v < vcount) union(u, v);
  }
  const membership = parent.map(find);
  const ids = new Set(membership);
  const remap = new Map<number, number>();
  let idx = 0;
  for (const id of ids) remap.set(id, idx++);
  return {
    membership: membership.map((m) => remap.get(m)!),
    count: ids.size,
  };
}

export function demoBetweenness(vcount: number, edges: Edge[]): AlgoResult {
  const adj = buildAdj(vcount, edges);
  const cb = new Float64Array(vcount);
  for (let s = 0; s < vcount; s++) {
    const stack: number[] = [];
    const pred: number[][] = Array.from({ length: vcount }, () => []);
    const sigma = new Float64Array(vcount);
    sigma[s] = 1;
    const dist = new Int32Array(vcount).fill(-1);
    dist[s] = 0;
    const queue = [s];
    while (queue.length > 0) {
      const v = queue.shift()!;
      stack.push(v);
      for (const w of adj[v]!) {
        if (dist[w]! < 0) {
          queue.push(w);
          dist[w] = dist[v]! + 1;
        }
        if (dist[w] === dist[v]! + 1) {
          sigma[w]! += sigma[v]!;
          pred[w]!.push(v);
        }
      }
    }
    const delta = new Float64Array(vcount);
    while (stack.length > 0) {
      const w = stack.pop()!;
      for (const v of pred[w]!) {
        delta[v]! += (sigma[v]! / sigma[w]!) * (1 + delta[w]!);
      }
      if (w !== s) cb[w]! += delta[w]!;
    }
  }
  return { scores: Array.from(cb) };
}

export function demoLouvain(vcount: number, edges: Edge[]): AlgoResult {
  const result = demoComponents(vcount, edges) as { membership: number[]; count: number };
  return { membership: result.membership, modularity: 0 };
}

export function demoInfomap(vcount: number, edges: Edge[]): AlgoResult {
  const result = demoComponents(vcount, edges) as { membership: number[]; count: number };
  return { membership: result.membership, codelength: 0 };
}

export function demoSpinglass(vcount: number, edges: Edge[]): AlgoResult {
  const result = demoComponents(vcount, edges) as { membership: number[]; count: number };
  return {
    membership: result.membership,
    modularity: 0,
    nb_clusters: result.count,
  };
}

export function demoCloseness(vcount: number, edges: Edge[]): AlgoResult {
  const scores = demoBetweenness(vcount, edges) as { scores: number[] };
  return { scores: scores.scores };
}

export function demoEigenvector(vcount: number, edges: Edge[]): AlgoResult {
  return demoPagerank(vcount, edges);
}

export function demoDfs(vcount: number, edges: Edge[], source = 0): AlgoResult {
  if (vcount === 0) return { order: [] };
  const start = source < vcount ? source : 0;
  const adj: number[][] = Array.from({ length: vcount }, () => []);
  for (const [u, v] of edges) {
    if (u < vcount && v < vcount) {
      adj[u]!.push(v);
      adj[v]!.push(u);
    }
  }
  const visited = new Set<number>();
  const order: number[] = [];
  const stack = [start];
  while (stack.length > 0) {
    const node = stack.pop()!;
    if (visited.has(node)) continue;
    visited.add(node);
    order.push(node);
    const neighbors = adj[node]!;
    for (let i = neighbors.length - 1; i >= 0; i--) {
      if (!visited.has(neighbors[i]!)) {
        stack.push(neighbors[i]!);
      }
    }
  }
  return { order };
}

export function runDemoAlgo(
  algo: string,
  vcount: number,
  edges: Edge[],
  params?: { damping?: number; source?: number },
): AlgoResult {
  switch (algo) {
    case 'bfs':
      return demoBfs(vcount, edges, params?.source);
    case 'dfs':
      return demoDfs(vcount, edges, params?.source);
    case 'pagerank':
      return demoPagerank(vcount, edges, params?.damping);
    case 'louvain':
      return demoLouvain(vcount, edges);
    case 'betweenness':
      return demoBetweenness(vcount, edges);
    case 'closeness':
      return demoCloseness(vcount, edges);
    case 'eigenvector':
      return demoEigenvector(vcount, edges);
    case 'components':
      return demoComponents(vcount, edges);
    case 'infomap':
      return demoInfomap(vcount, edges);
    case 'spinglass':
      return demoSpinglass(vcount, edges);
    default:
      return demoPagerank(vcount, edges);
  }
}
