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

export function demoLabelPropagation(vcount: number, edges: Edge[]): AlgoResult {
  const result = demoComponents(vcount, edges) as { membership: number[]; count: number };
  return { membership: result.membership, nb_clusters: result.count };
}

export function demoWalktrap(vcount: number, edges: Edge[]): AlgoResult {
  const result = demoComponents(vcount, edges) as { membership: number[]; count: number };
  return { membership: result.membership, nb_clusters: result.count, modularity: 0 };
}

export function demoLeiden(vcount: number, edges: Edge[]): AlgoResult {
  const result = demoComponents(vcount, edges) as { membership: number[]; count: number };
  return { membership: result.membership, nb_clusters: result.count, quality: 0 };
}

export function demoFastGreedy(vcount: number, edges: Edge[]): AlgoResult {
  const result = demoComponents(vcount, edges) as { membership: number[]; count: number };
  return { membership: result.membership, nb_clusters: result.count, modularity: 0 };
}

export function demoLeadingEigenvector(vcount: number, edges: Edge[]): AlgoResult {
  const result = demoComponents(vcount, edges) as { membership: number[]; count: number };
  return { membership: result.membership, modularity: 0 };
}

export function demoEdgeBetweennessCommunity(vcount: number, edges: Edge[]): AlgoResult {
  const result = demoComponents(vcount, edges) as { membership: number[]; count: number };
  return { membership: result.membership, nb_clusters: result.count };
}

export function demoFluid(vcount: number, edges: Edge[]): AlgoResult {
  const result = demoComponents(vcount, edges) as { membership: number[]; count: number };
  return { membership: result.membership, nb_clusters: result.count };
}

export function demoHarmonic(vcount: number, edges: Edge[]): AlgoResult {
  const adj = buildAdj(vcount, edges);
  const scores: number[] = [];
  for (let s = 0; s < vcount; s++) {
    const dist = new Int32Array(vcount).fill(-1);
    dist[s] = 0;
    const queue = [s];
    while (queue.length > 0) {
      const v = queue.shift()!;
      for (const w of adj[v]!) {
        if (dist[w]! < 0) {
          dist[w] = dist[v]! + 1;
          queue.push(w);
        }
      }
    }
    let sum = 0;
    for (let t = 0; t < vcount; t++) {
      if (t !== s && dist[t]! > 0) sum += 1 / dist[t]!;
    }
    scores.push(sum);
  }
  return { scores };
}

export function demoHits(vcount: number, edges: Edge[]): AlgoResult {
  const adj = buildAdj(vcount, edges);
  let auth = new Float64Array(vcount).fill(1);
  let hub = new Float64Array(vcount).fill(1);
  for (let iter = 0; iter < 50; iter++) {
    const newAuth = new Float64Array(vcount);
    for (let v = 0; v < vcount; v++) {
      for (const u of adj[v]!) newAuth[v]! += hub[u]!;
    }
    let norm = Math.sqrt(newAuth.reduce((s, x) => s + x * x, 0)) || 1;
    for (let i = 0; i < vcount; i++) newAuth[i]! /= norm;

    const newHub = new Float64Array(vcount);
    for (let v = 0; v < vcount; v++) {
      for (const u of adj[v]!) newHub[v]! += newAuth[u]!;
    }
    norm = Math.sqrt(newHub.reduce((s, x) => s + x * x, 0)) || 1;
    for (let i = 0; i < vcount; i++) newHub[i]! /= norm;

    auth = newAuth;
    hub = newHub;
  }
  return { hub: Array.from(hub), authority: Array.from(auth) };
}

export function demoKatz(vcount: number, edges: Edge[]): AlgoResult {
  const adj = buildAdj(vcount, edges);
  const alpha = 0.01;
  const beta = 1.0;
  const scores = new Float64Array(vcount).fill(beta);
  for (let iter = 0; iter < 50; iter++) {
    const next = new Float64Array(vcount).fill(beta);
    for (let v = 0; v < vcount; v++) {
      for (const u of adj[v]!) {
        next[v]! += alpha * scores[u]!;
      }
    }
    scores.set(next);
  }
  return { scores: Array.from(scores) };
}

export function demoDijkstra(vcount: number, edges: Edge[], source = 0): AlgoResult {
  const adj = buildAdj(vcount, edges);
  const start = source >= 0 && source < vcount ? source : 0;
  const dist = new Float64Array(vcount).fill(Infinity);
  dist[start] = 0;
  const visited = new Set<number>();
  for (let i = 0; i < vcount; i++) {
    let u = -1;
    for (let v = 0; v < vcount; v++) {
      if (!visited.has(v) && (u < 0 || dist[v]! < dist[u]!)) u = v;
    }
    if (u < 0 || dist[u] === Infinity) break;
    visited.add(u);
    for (const w of adj[u]!) {
      const nd = dist[u]! + 1;
      if (nd < dist[w]!) dist[w] = nd;
    }
  }
  return { distances: Array.from(dist) } as AlgoResult;
}

export function demoGraphStats(vcount: number, edges: Edge[]): AlgoResult {
  const adj = buildAdj(vcount, edges);
  const visited = new Set<number>();
  const queue = [0];
  if (vcount > 0) visited.add(0);
  while (queue.length > 0) {
    const v = queue.shift()!;
    for (const w of adj[v]!) {
      if (!visited.has(w)) { visited.add(w); queue.push(w); }
    }
  }
  return {
    vcount,
    ecount: edges.length,
    is_directed: false,
    is_connected: visited.size === vcount,
    diameter: 0,
    girth: 0,
    triangles: 0,
    is_bipartite: false,
  } as AlgoResult;
}

export function demoMaxFlow(vcount: number, _edges: Edge[]): AlgoResult {
  return { value: vcount > 0 ? 1 : 0 } as AlgoResult;
}

export function demoArticulationPoints(vcount: number, edges: Edge[]): AlgoResult {
  const adj = buildAdj(vcount, edges);
  const points: number[] = [];
  for (let r = 0; r < vcount; r++) {
    const visited = new Set<number>();
    const start = r === 0 ? 1 : 0;
    if (start >= vcount) continue;
    visited.add(r);
    visited.add(start);
    const q = [start];
    while (q.length > 0) {
      const v = q.shift()!;
      for (const w of adj[v]!) {
        if (!visited.has(w)) { visited.add(w); q.push(w); }
      }
    }
    if (visited.size < vcount) points.push(r);
  }
  return { vertices: points } as AlgoResult;
}

export function demoDegreeSequence(vcount: number, edges: Edge[]): AlgoResult {
  const degrees = new Array(vcount).fill(0);
  for (const [u, v] of edges) {
    if (u < vcount) degrees[u]++;
    if (v < vcount) degrees[v]++;
  }
  return { degrees } as AlgoResult;
}

export function demoScc(vcount: number, edges: Edge[]): AlgoResult {
  const result = demoComponents(vcount, edges) as { membership: number[]; count: number };
  return { membership: result.membership, count: result.count };
}

export function demoBridges(vcount: number, edges: Edge[]): AlgoResult {
  const adj = buildAdj(vcount, edges);
  const disc = new Int32Array(vcount).fill(-1);
  const low = new Int32Array(vcount).fill(-1);
  const bridgeEdges: [number, number][] = [];
  let timer = 0;

  function dfs(u: number, parent: number) {
    disc[u] = low[u] = timer++;
    for (const v of adj[u]!) {
      if (v === parent) continue;
      if (disc[v]! < 0) {
        dfs(v, u);
        low[u] = Math.min(low[u]!, low[v]!);
        if (low[v]! > disc[u]!) bridgeEdges.push([u, v]);
      } else {
        low[u] = Math.min(low[u]!, disc[v]!);
      }
    }
  }

  for (let i = 0; i < vcount; i++) {
    if (disc[i]! < 0) dfs(i, -1);
  }
  return { edges: bridgeEdges, count: bridgeEdges.length } as AlgoResult;
}

export function demoColoring(vcount: number, edges: Edge[]): AlgoResult {
  const adj = buildAdj(vcount, edges);
  const colors = new Array<number>(vcount).fill(-1);
  for (let v = 0; v < vcount; v++) {
    const used = new Set<number>();
    for (const u of adj[v]!) {
      if (colors[u]! >= 0) used.add(colors[u]!);
    }
    let c = 0;
    while (used.has(c)) c++;
    colors[v] = c;
  }
  const chromatic = colors.length > 0 ? Math.max(...colors) + 1 : 0;
  return { colors, chromatic } as AlgoResult;
}

export function demoTopologicalSort(vcount: number, edges: Edge[]): AlgoResult {
  const inDeg = new Int32Array(vcount);
  const adj: number[][] = Array.from({ length: vcount }, () => []);
  for (const [u, v] of edges) {
    if (u < vcount && v < vcount) {
      adj[u]!.push(v);
      inDeg[v]!++;
    }
  }
  const queue: number[] = [];
  for (let i = 0; i < vcount; i++) {
    if (inDeg[i] === 0) queue.push(i);
  }
  const order: number[] = [];
  while (queue.length > 0) {
    const v = queue.shift()!;
    order.push(v);
    for (const w of adj[v]!) {
      inDeg[w]!--;
      if (inDeg[w] === 0) queue.push(w);
    }
  }
  return { order };
}

export function demoTransitivity(vcount: number, edges: Edge[]): AlgoResult {
  const adj = buildAdj(vcount, edges);
  let triangles = 0;
  let triples = 0;
  for (let v = 0; v < vcount; v++) {
    const nbrs = adj[v]!;
    const nbrSet = new Set(nbrs);
    for (let i = 0; i < nbrs.length; i++) {
      for (let j = i + 1; j < nbrs.length; j++) {
        triples++;
        if (nbrSet.has(nbrs[j]!) && adj[nbrs[i]!]!.includes(nbrs[j]!)) triangles++;
      }
    }
  }
  return { value: triples > 0 ? triangles / triples : 0 } as AlgoResult;
}

export function demoTriadCensus(vcount: number, edges: Edge[]): AlgoResult {
  const counts = new Array(16).fill(0);
  const triples = vcount * (vcount - 1) * (vcount - 2) / 6;
  counts[0] = Math.max(0, triples - edges.length);
  counts[1] = edges.length;
  return { counts } as AlgoResult;
}

export function demoCanonicalPermutation(vcount: number, _edges: Edge[]): AlgoResult {
  const permutation = Array.from({ length: vcount }, (_, i) => i);
  return { permutation } as AlgoResult;
}

export function demoCountAutomorphisms(_vcount: number, _edges: Edge[]): AlgoResult {
  return { count: 1 } as AlgoResult;
}

export function demoIsomorphism(vcount: number, _edges: Edge[]): AlgoResult {
  const mapping = Array.from({ length: vcount }, (_, i) => i);
  return { isomorphic: true, mapping } as AlgoResult;
}

export function demoCoreness(vcount: number, edges: Edge[]): AlgoResult {
  const deg = new Array(vcount).fill(0);
  for (const [u, v] of edges) {
    if (u < vcount && v < vcount) { deg[u]++; deg[v]++; }
  }
  const cores = new Array(vcount).fill(0);
  const removed = new Set<number>();
  let k = 1;
  while (removed.size < vcount) {
    let changed = true;
    while (changed) {
      changed = false;
      for (let v = 0; v < vcount; v++) {
        if (!removed.has(v) && deg[v]! < k) {
          removed.add(v);
          cores[v] = k - 1;
          for (const [u, w] of edges) {
            if (u === v && w < vcount && !removed.has(w)) deg[w]!--;
            if (w === v && u < vcount && !removed.has(u)) deg[u]!--;
          }
          changed = true;
        }
      }
    }
    k++;
  }
  return { cores } as AlgoResult;
}

export function demoEccentricity(vcount: number, edges: Edge[]): AlgoResult {
  const adj = buildAdj(vcount, edges);
  const values: number[] = [];
  for (let s = 0; s < vcount; s++) {
    const dist = new Int32Array(vcount).fill(-1);
    dist[s] = 0;
    const queue = [s];
    let maxDist = 0;
    while (queue.length > 0) {
      const v = queue.shift()!;
      for (const w of adj[v]!) {
        if (dist[w]! < 0) {
          dist[w] = dist[v]! + 1;
          if (dist[w]! > maxDist) maxDist = dist[w]!;
          queue.push(w);
        }
      }
    }
    values.push(maxDist);
  }
  return { values };
}

export function demoConstraint(vcount: number, edges: Edge[]): AlgoResult {
  const adj = buildAdj(vcount, edges);
  const scores: number[] = [];
  for (let v = 0; v < vcount; v++) {
    const nbrs = adj[v]!;
    if (nbrs.length === 0) { scores.push(0); continue; }
    const nbrSet = new Set(nbrs);
    let constraint = 0;
    for (const j of nbrs) {
      let pij = 1 / nbrs.length;
      for (const q of nbrs) {
        if (q !== j && adj[q]!.includes(j)) {
          pij += (1 / nbrs.length) * (1 / (nbrSet.has(q) ? adj[q]!.filter(x => nbrSet.has(x) || x === v).length : 1));
        }
      }
      constraint += pij * pij;
    }
    scores.push(constraint);
  }
  return { scores } as AlgoResult;
}

export function demoDiameter(vcount: number, edges: Edge[]): AlgoResult {
  const adj = buildAdj(vcount, edges);
  let maxDist = 0;
  for (let s = 0; s < vcount; s++) {
    const dist = new Int32Array(vcount).fill(-1);
    dist[s] = 0;
    const queue = [s];
    while (queue.length > 0) {
      const v = queue.shift()!;
      for (const w of adj[v]!) {
        if (dist[w]! < 0) {
          dist[w] = dist[v]! + 1;
          if (dist[w]! > maxDist) maxDist = dist[w]!;
          queue.push(w);
        }
      }
    }
  }
  return { diameter: vcount > 0 ? maxDist : null } as AlgoResult;
}

export function demoShortestPath(vcount: number, edges: Edge[], source = 0, target = 1): AlgoResult {
  if (vcount === 0) return { path: [] } as AlgoResult;
  const adj = buildAdj(vcount, edges);
  const src = source >= 0 && source < vcount ? source : 0;
  const tgt = target >= 0 && target < vcount ? target : Math.min(1, vcount - 1);
  const prev = new Int32Array(vcount).fill(-1);
  const visited = new Set<number>();
  visited.add(src);
  const queue = [src];
  while (queue.length > 0) {
    const v = queue.shift()!;
    if (v === tgt) break;
    for (const w of adj[v]!) {
      if (!visited.has(w)) {
        visited.add(w);
        prev[w] = v;
        queue.push(w);
      }
    }
  }
  if (!visited.has(tgt)) return { path: [] } as AlgoResult;
  const path: number[] = [];
  let cur = tgt;
  while (cur !== src) {
    path.unshift(cur);
    cur = prev[cur]!;
  }
  path.unshift(src);
  return { path } as AlgoResult;
}

export function demoRandomWalk(vcount: number, edges: Edge[], source = 0, steps = 20): AlgoResult {
  if (vcount === 0) return { vertices: [] } as AlgoResult;
  const adj = buildAdj(vcount, edges);
  const start = source >= 0 && source < vcount ? source : 0;
  const vertices: number[] = [start];
  let cur = start;
  let seed = 42;
  for (let i = 0; i < steps; i++) {
    const nbrs = adj[cur]!;
    if (nbrs.length === 0) break;
    seed = (seed * 1103515245 + 12345) & 0x7fffffff;
    cur = nbrs[seed % nbrs.length]!;
    vertices.push(cur);
  }
  return { vertices } as AlgoResult;
}

export function demoEdgeBetweennessCentrality(vcount: number, edges: Edge[]): AlgoResult {
  const scores = new Array(edges.length).fill(0);
  const adj = buildAdj(vcount, edges);
  const edgeIndex = new Map<string, number>();
  edges.forEach(([u, v], i) => {
    edgeIndex.set(`${Math.min(u, v)}-${Math.max(u, v)}`, i);
  });

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
        if (dist[w]! < 0) { queue.push(w); dist[w] = dist[v]! + 1; }
        if (dist[w] === dist[v]! + 1) { sigma[w]! += sigma[v]!; pred[w]!.push(v); }
      }
    }
    const delta = new Float64Array(vcount);
    while (stack.length > 0) {
      const w = stack.pop()!;
      for (const v of pred[w]!) {
        const c = (sigma[v]! / sigma[w]!) * (1 + delta[w]!);
        delta[v]! += c;
        const key = `${Math.min(v, w)}-${Math.max(v, w)}`;
        const idx = edgeIndex.get(key);
        if (idx !== undefined) scores[idx]! += c;
      }
    }
  }
  return { scores } as AlgoResult;
}

export function demoFundamentalCycles(vcount: number, edges: Edge[]): AlgoResult {
  const adj: number[][] = Array.from({ length: vcount }, () => []);
  for (const [u, v] of edges) { adj[u]!.push(v); adj[v]!.push(u); }
  const visited = new Uint8Array(vcount);
  const parent = new Int32Array(vcount).fill(-1);
  const cycles: number[][] = [];
  const queue: number[] = [0];
  visited[0] = 1;
  const edgeSet = new Set<string>();
  while (queue.length > 0) {
    const u = queue.shift()!;
    for (const v of adj[u]!) {
      const key = `${Math.min(u, v)}-${Math.max(u, v)}`;
      if (!visited[v]) {
        visited[v] = 1;
        parent[v] = u;
        queue.push(v);
        edgeSet.add(key);
      } else if (!edgeSet.has(key) && parent[u] !== v) {
        const cycle: number[] = [];
        let a = u, b = v;
        const pathA: number[] = [];
        const pathB: number[] = [];
        while (a !== -1) { pathA.push(a); a = parent[a]!; }
        while (b !== -1) { pathB.push(b); b = parent[b]!; }
        const setA = new Set(pathA);
        let lca = pathB.find(x => setA.has(x)) ?? 0;
        for (const x of pathA) { cycle.push(x); if (x === lca) break; }
        const bPart: number[] = [];
        for (const x of pathB) { if (x === lca) break; bPart.push(x); }
        cycle.push(...bPart.reverse());
        cycles.push(cycle);
        edgeSet.add(key);
      }
    }
  }
  return { cycles, count: cycles.length } as AlgoResult;
}

export function demoListTriangles(vcount: number, edges: Edge[]): AlgoResult {
  const adj: Set<number>[] = Array.from({ length: vcount }, () => new Set());
  for (const [u, v] of edges) { adj[u]!.add(v); adj[v]!.add(u); }
  const triangles: [number, number, number][] = [];
  for (const [u, v] of edges) {
    for (const w of adj[u]!) {
      if (w > u && w > v && adj[v]!.has(w)) {
        const sorted = [u, v, w].sort((a, b) => a - b) as [number, number, number];
        triangles.push(sorted);
      }
    }
  }
  const seen = new Set<string>();
  const unique = triangles.filter(t => {
    const k = t.join('-');
    if (seen.has(k)) return false;
    seen.add(k);
    return true;
  });
  return { triangles: unique, count: unique.length } as AlgoResult;
}

export function demoGirth(vcount: number, edges: Edge[]): AlgoResult {
  const adj: number[][] = Array.from({ length: vcount }, () => []);
  for (const [u, v] of edges) { adj[u]!.push(v); adj[v]!.push(u); }
  let girth: number | null = null;
  for (let s = 0; s < vcount; s++) {
    const dist = new Int32Array(vcount).fill(-1);
    dist[s] = 0;
    const queue: number[] = [s];
    while (queue.length > 0) {
      const u = queue.shift()!;
      for (const v of adj[u]!) {
        if (dist[v]! < 0) {
          dist[v] = dist[u]! + 1;
          queue.push(v);
        } else if (dist[v]! >= dist[u]!) {
          const len = dist[u]! + dist[v]! + 1;
          if (girth === null || len < girth) girth = len;
        }
      }
    }
  }
  return { diameter: girth } as AlgoResult;
}

export function demoTrussness(vcount: number, edges: Edge[]): AlgoResult {
  const adj: Set<number>[] = Array.from({ length: vcount }, () => new Set());
  for (const [u, v] of edges) { adj[u]!.add(v); adj[v]!.add(u); }
  const trussness: number[] = edges.map(([u, v]) => {
    let support = 0;
    for (const w of adj[u]!) {
      if (adj[v]!.has(w)) support++;
    }
    return support + 2;
  });
  return { trussness } as AlgoResult;
}

export function demoAutomorphismGroup(vcount: number, _edges: Edge[]): AlgoResult {
  const generators: number[][] = [];
  if (vcount >= 2) {
    const gen = Array.from({ length: vcount }, (_, i) => i);
    gen[0] = 1;
    gen[1] = 0;
    generators.push(gen);
  }
  return { generators, count: generators.length } as AlgoResult;
}

export function demoCliqueNumber(vcount: number, edges: Edge[]): AlgoResult {
  const adj: Set<number>[] = Array.from({ length: vcount }, () => new Set());
  for (const [u, v] of edges) {
    adj[u]!.add(v);
    adj[v]!.add(u);
  }
  let maxClique = vcount > 0 ? 1 : 0;
  for (const [u, v] of edges) {
    let triSize = 2;
    for (const w of adj[u]!) {
      if (adj[v]!.has(w)) triSize = 3;
    }
    if (triSize > maxClique) maxClique = triSize;
  }
  return { value: maxClique } as AlgoResult;
}

export function demoIndependenceNumber(vcount: number, edges: Edge[]): AlgoResult {
  const adj: Set<number>[] = Array.from({ length: vcount }, () => new Set());
  for (const [u, v] of edges) {
    adj[u]!.add(v);
    adj[v]!.add(u);
  }
  let maxIndep = 0;
  for (let mask = 0; mask < (1 << Math.min(vcount, 16)); mask++) {
    const verts: number[] = [];
    for (let i = 0; i < Math.min(vcount, 16); i++) {
      if (mask & (1 << i)) verts.push(i);
    }
    let independent = true;
    outer: for (let i = 0; i < verts.length; i++) {
      for (let j = i + 1; j < verts.length; j++) {
        if (adj[verts[i]!]!.has(verts[j]!)) { independent = false; break outer; }
      }
    }
    if (independent && verts.length > maxIndep) maxIndep = verts.length;
  }
  return { value: maxIndep } as AlgoResult;
}

export function demoMaximalCliques(vcount: number, edges: Edge[]): AlgoResult {
  const adj: Set<number>[] = Array.from({ length: vcount }, () => new Set());
  for (const [u, v] of edges) {
    adj[u]!.add(v);
    adj[v]!.add(u);
  }
  const cliques: number[][] = [];
  function bronKerbosch(R: Set<number>, P: Set<number>, X: Set<number>) {
    if (P.size === 0 && X.size === 0) {
      cliques.push([...R].sort((a, b) => a - b));
      return;
    }
    const pivot = [...P, ...X][0]!;
    const candidates = [...P].filter(v => !adj[pivot]!.has(v));
    for (const v of candidates) {
      const newR = new Set(R); newR.add(v);
      const newP = new Set([...P].filter(u => adj[v]!.has(u)));
      const newX = new Set([...X].filter(u => adj[v]!.has(u)));
      bronKerbosch(newR, newP, newX);
      P.delete(v);
      X.add(v);
    }
  }
  bronKerbosch(new Set(), new Set(Array.from({ length: vcount }, (_, i) => i)), new Set());
  return { cliques, count: cliques.length } as AlgoResult;
}

export function demoVertexConnectivity(vcount: number, edges: Edge[]): AlgoResult {
  if (vcount <= 1) return { value: 0 } as AlgoResult;
  let minCut = vcount;
  for (let v = 0; v < vcount; v++) {
    let deg = 0;
    for (const [u, w] of edges) {
      if (u === v || w === v) deg++;
    }
    if (deg < minCut) minCut = deg;
  }
  return { value: minCut } as AlgoResult;
}

export function demoEdgeConnectivity(vcount: number, edges: Edge[]): AlgoResult {
  if (vcount <= 1) return { value: 0 } as AlgoResult;
  let minDeg = edges.length;
  for (let v = 0; v < vcount; v++) {
    let deg = 0;
    for (const [u, w] of edges) {
      if (u === v || w === v) deg++;
    }
    if (deg < minDeg) minDeg = deg;
  }
  return { value: Math.min(minDeg, edges.length) } as AlgoResult;
}

export function demoMinimumSpanningTree(vcount: number, edges: Edge[]): AlgoResult {
  const parent = Array.from({ length: vcount }, (_, i) => i);
  function find(x: number): number {
    while (parent[x] !== x) { parent[x] = parent[parent[x]!]!; x = parent[x]!; }
    return x;
  }
  const mstEdges: number[] = [];
  for (let i = 0; i < edges.length; i++) {
    const [u, v] = edges[i]!;
    const ru = find(u);
    const rv = find(v);
    if (ru !== rv) {
      parent[ru] = rv;
      mstEdges.push(i);
    }
  }
  return { edges: mstEdges, count: mstEdges.length } as AlgoResult;
}

export function demoBellmanFord(vcount: number, edges: Edge[], source = 0): AlgoResult {
  const dist: (number | null)[] = Array.from({ length: vcount }, () => null);
  dist[source] = 0;
  for (let i = 0; i < vcount - 1; i++) {
    for (const [u, v] of edges) {
      if (dist[u] !== null && (dist[v] === null || dist[u]! + 1 < dist[v]!)) {
        dist[v] = dist[u]! + 1;
      }
      if (dist[v] !== null && (dist[u] === null || dist[v]! + 1 < dist[u]!)) {
        dist[u] = dist[v]! + 1;
      }
    }
  }
  return { distances: dist } as AlgoResult;
}

export function demoDegreeDistribution(vcount: number, edges: Edge[]): AlgoResult {
  const degrees = new Array(vcount).fill(0);
  for (const [u, v] of edges) {
    if (u < vcount) degrees[u]++;
    if (v < vcount) degrees[v]++;
  }
  return { degrees } as AlgoResult;
}

export function demoFeedbackArcSet(_vcount: number, edges: Edge[]): AlgoResult {
  const edgeIndices: number[] = [];
  for (let i = 0; i < edges.length; i++) {
    const [u, v] = edges[i]!;
    if (u > v) edgeIndices.push(i);
  }
  return { edges: edgeIndices, count: edgeIndices.length } as AlgoResult;
}

export function demoMinimumCycleBasis(vcount: number, edges: Edge[]): AlgoResult {
  const cycles: number[][] = [];
  const adj = new Array(vcount).fill(null).map(() => [] as number[]);
  for (const [u, v] of edges) {
    if (u < vcount && v < vcount) {
      adj[u]!.push(v);
      adj[v]!.push(u);
    }
  }
  for (let i = 0; i < vcount; i++) {
    for (const j of adj[i]!) {
      for (const k of adj[j]!) {
        if (k > i && adj[k]!.includes(i)) {
          cycles.push([i, j, k]);
        }
      }
    }
  }
  return { cycles, count: cycles.length } as AlgoResult;
}

export function demoBiconnectedComponents(vcount: number, edges: Edge[]): AlgoResult {
  const adj: { to: number; idx: number }[][] = Array.from({ length: vcount }, () => []);
  for (let i = 0; i < edges.length; i++) {
    const [u, v] = edges[i]!;
    if (u < vcount && v < vcount) {
      adj[u]!.push({ to: v, idx: i });
      adj[v]!.push({ to: u, idx: i });
    }
  }
  const disc = new Int32Array(vcount).fill(-1);
  const low = new Int32Array(vcount).fill(-1);
  const components: number[][] = [];
  const stack: number[] = [];
  let timer = 0;

  function dfs(u: number, parentEdge: number) {
    disc[u] = low[u] = timer++;
    for (const { to: v, idx } of adj[u]!) {
      if (idx === parentEdge) continue;
      if (disc[v]! < 0) {
        stack.push(idx);
        dfs(v, idx);
        low[u] = Math.min(low[u]!, low[v]!);
        if (low[v]! >= disc[u]!) {
          const comp: number[] = [];
          while (stack.length > 0) {
            const e = stack.pop()!;
            comp.push(e);
            if (e === idx) break;
          }
          components.push(comp);
        }
      } else if (disc[v]! < disc[u]!) {
        stack.push(idx);
        low[u] = Math.min(low[u]!, disc[v]!);
      }
    }
  }

  for (let i = 0; i < vcount; i++) {
    if (disc[i]! < 0) dfs(i, -1);
  }
  return { count: components.length, components } as AlgoResult;
}

export function demoBipartiteCheck(vcount: number, edges: Edge[]): AlgoResult {
  const adj = buildAdj(vcount, edges);
  const color = new Int32Array(vcount).fill(-1);
  let bipartite = true;
  for (let s = 0; s < vcount && bipartite; s++) {
    if (color[s]! >= 0) continue;
    color[s] = 0;
    const queue = [s];
    while (queue.length > 0 && bipartite) {
      const u = queue.shift()!;
      for (const v of adj[u]!) {
        if (color[v]! < 0) {
          color[v] = 1 - color[u]!;
          queue.push(v);
        } else if (color[v] === color[u]) {
          bipartite = false;
        }
      }
    }
  }
  return { is_bipartite: bipartite, types: Array.from(color).map(c => (c < 0 ? 0 : c)) } as AlgoResult;
}

export function demoMaximumCut(vcount: number, edges: Edge[]): AlgoResult {
  const partition = new Array<boolean>(vcount).fill(false);
  for (let v = 0; v < vcount; v++) {
    let crossTrue = 0;
    let crossFalse = 0;
    for (const [u, w] of edges) {
      if (u === v && w < v) {
        if (partition[w]) crossFalse++;
        else crossTrue++;
      }
      if (w === v && u < v) {
        if (partition[u]) crossFalse++;
        else crossTrue++;
      }
    }
    partition[v] = crossTrue >= crossFalse;
  }
  let cutValue = 0;
  for (const [u, v] of edges) {
    if (u < vcount && v < vcount && partition[u] !== partition[v]) cutValue++;
  }
  return { partition, cut_value: cutValue } as AlgoResult;
}

export function demoGlobalEfficiency(vcount: number, edges: Edge[]): AlgoResult {
  if (vcount <= 1) return { value: 0 } as AlgoResult;
  const adj = buildAdj(vcount, edges);
  let totalInvDist = 0;
  for (let s = 0; s < vcount; s++) {
    const dist = new Int32Array(vcount).fill(-1);
    dist[s] = 0;
    const queue = [s];
    while (queue.length > 0) {
      const v = queue.shift()!;
      for (const w of adj[v]!) {
        if (dist[w]! < 0) {
          dist[w] = dist[v]! + 1;
          queue.push(w);
        }
      }
    }
    for (let t = 0; t < vcount; t++) {
      if (t !== s && dist[t]! > 0) totalInvDist += 1 / dist[t]!;
    }
  }
  return { value: totalInvDist / (vcount * (vcount - 1)) } as AlgoResult;
}

export function demoLocalEfficiency(vcount: number, edges: Edge[]): AlgoResult {
  const adj = buildAdj(vcount, edges);
  const scores: number[] = [];
  for (let v = 0; v < vcount; v++) {
    const nbrs = adj[v]!;
    if (nbrs.length <= 1) { scores.push(0); continue; }
    const nbrSet = new Set(nbrs);
    const subAdj = new Map<number, number[]>();
    for (const u of nbrs) {
      subAdj.set(u, adj[u]!.filter(w => nbrSet.has(w) && w !== v));
    }
    let totalInv = 0;
    for (const s of nbrs) {
      const dist = new Map<number, number>();
      dist.set(s, 0);
      const queue = [s];
      while (queue.length > 0) {
        const u = queue.shift()!;
        for (const w of (subAdj.get(u) ?? [])) {
          if (!dist.has(w)) {
            dist.set(w, dist.get(u)! + 1);
            queue.push(w);
          }
        }
      }
      for (const t of nbrs) {
        if (t !== s) {
          const d = dist.get(t);
          if (d !== undefined && d > 0) totalInv += 1 / d;
        }
      }
    }
    const n = nbrs.length;
    scores.push(totalInv / (n * (n - 1)));
  }
  return { scores } as AlgoResult;
}

export function demoDegeneracy(vcount: number, edges: Edge[]): AlgoResult {
  const deg = new Array(vcount).fill(0);
  for (const [u, v] of edges) {
    if (u < vcount && v < vcount) { deg[u]++; deg[v]++; }
  }
  const removed = new Set<number>();
  let maxCore = 0;
  let k = 1;
  while (removed.size < vcount) {
    let changed = true;
    while (changed) {
      changed = false;
      for (let v = 0; v < vcount; v++) {
        if (!removed.has(v) && deg[v]! < k) {
          removed.add(v);
          for (const [u, w] of edges) {
            if (u === v && w < vcount && !removed.has(w)) deg[w]!--;
            if (w === v && u < vcount && !removed.has(u)) deg[u]!--;
          }
          changed = true;
          if (k - 1 > maxCore) maxCore = k - 1;
        }
      }
    }
    k++;
  }
  return { value: maxCore } as AlgoResult;
}

export function demoAllSimplePaths(vcount: number, edges: Edge[], source = 0, target = 1): AlgoResult {
  if (vcount === 0) return { paths: [], count: 0 } as AlgoResult;
  const adj = buildAdj(vcount, edges);
  const src = source >= 0 && source < vcount ? source : 0;
  const tgt = target >= 0 && target < vcount ? target : Math.min(1, vcount - 1);
  const paths: number[][] = [];
  const visited = new Set<number>();

  function dfs(u: number, path: number[]) {
    if (paths.length >= 100) return;
    if (u === tgt) { paths.push([...path]); return; }
    for (const w of adj[u]!) {
      if (!visited.has(w)) {
        visited.add(w);
        path.push(w);
        dfs(w, path);
        path.pop();
        visited.delete(w);
      }
    }
  }

  visited.add(src);
  dfs(src, [src]);
  return { paths, count: paths.length } as AlgoResult;
}

export function demoFindCycle(vcount: number, edges: Edge[]): AlgoResult {
  const adj = buildAdj(vcount, edges);
  const color = new Uint8Array(vcount);
  const parent = new Int32Array(vcount).fill(-1);
  let cycleVerts: number[] = [];
  let found = false;

  function dfs(u: number, par: number): boolean {
    color[u] = 1;
    for (const v of adj[u]!) {
      if (v === par) continue;
      if (color[v] === 1) {
        const cycle: number[] = [v];
        let cur = u;
        while (cur !== v) {
          cycle.push(cur);
          cur = parent[cur]!;
        }
        cycle.push(v);
        cycleVerts = cycle;
        return true;
      }
      if (color[v] === 0) {
        parent[v] = u;
        if (dfs(v, u)) return true;
      }
    }
    color[u] = 2;
    return false;
  }

  for (let i = 0; i < vcount && !found; i++) {
    if (color[i] === 0) {
      found = dfs(i, -1);
    }
  }

  const cycleEdges: number[] = [];
  if (found && cycleVerts.length > 1) {
    for (let i = 0; i < cycleVerts.length - 1; i++) {
      const u = cycleVerts[i]!;
      const v = cycleVerts[i + 1]!;
      const idx = edges.findIndex(([a, b]) =>
        (a === u && b === v) || (a === v && b === u)
      );
      if (idx >= 0) cycleEdges.push(idx);
    }
  }
  return { vertices: cycleVerts, edges: cycleEdges, found } as AlgoResult;
}

export function demoMincutValue(vcount: number, edges: Edge[]): AlgoResult {
  if (vcount <= 1) return { value: 0 } as AlgoResult;
  let minDeg = edges.length;
  const adj = buildAdj(vcount, edges);
  for (let v = 0; v < vcount; v++) {
    if (adj[v]!.length < minDeg) minDeg = adj[v]!.length;
  }
  return { value: Math.min(minDeg, edges.length) } as AlgoResult;
}

export function demoVertexDisjointPaths(vcount: number, edges: Edge[], source = 0, target = 1): AlgoResult {
  if (vcount === 0) return { value: 0 } as AlgoResult;
  const adj = buildAdj(vcount, edges);
  const src = source >= 0 && source < vcount ? source : 0;
  const tgt = target >= 0 && target < vcount ? target : Math.min(1, vcount - 1);
  if (src === tgt) return { value: 0 } as AlgoResult;
  let count = 0;
  const globalBlocked = new Set<number>();
  for (let iter = 0; iter < vcount; iter++) {
    const visited = new Set<number>();
    visited.add(src);
    for (const b of globalBlocked) visited.add(b);
    const prev = new Int32Array(vcount).fill(-1);
    const queue = [src];
    let found = false;
    while (queue.length > 0 && !found) {
      const u = queue.shift()!;
      for (const w of adj[u]!) {
        if (!visited.has(w)) {
          visited.add(w);
          prev[w] = u;
          if (w === tgt) { found = true; break; }
          queue.push(w);
        }
      }
    }
    if (!found) break;
    count++;
    let cur = tgt;
    while (cur !== src) {
      if (cur !== tgt) globalBlocked.add(cur);
      cur = prev[cur]!;
    }
  }
  return { value: count } as AlgoResult;
}

export function demoEdgeDisjointPaths(vcount: number, edges: Edge[], source = 0, target = 1): AlgoResult {
  if (vcount === 0) return { value: 0 } as AlgoResult;
  const src = source >= 0 && source < vcount ? source : 0;
  const tgt = target >= 0 && target < vcount ? target : Math.min(1, vcount - 1);
  if (src === tgt) return { value: 0 } as AlgoResult;
  const capacity = new Array(edges.length).fill(1);
  let count = 0;
  for (let iter = 0; iter < edges.length; iter++) {
    const adj: { to: number; eIdx: number }[][] = Array.from({ length: vcount }, () => []);
    for (let i = 0; i < edges.length; i++) {
      if (capacity[i]! <= 0) continue;
      const [u, v] = edges[i]!;
      if (u < vcount && v < vcount) {
        adj[u]!.push({ to: v, eIdx: i });
        adj[v]!.push({ to: u, eIdx: i });
      }
    }
    const visited = new Set<number>();
    visited.add(src);
    const prev: { node: number; eIdx: number }[] = Array.from({ length: vcount }, () => ({ node: -1, eIdx: -1 }));
    const queue = [src];
    let found = false;
    while (queue.length > 0 && !found) {
      const u = queue.shift()!;
      for (const { to: w, eIdx } of adj[u]!) {
        if (!visited.has(w)) {
          visited.add(w);
          prev[w] = { node: u, eIdx };
          if (w === tgt) { found = true; break; }
          queue.push(w);
        }
      }
    }
    if (!found) break;
    count++;
    let cur = tgt;
    while (cur !== src) {
      capacity[prev[cur]!.eIdx]!--;
      cur = prev[cur]!.node;
    }
  }
  return { value: count } as AlgoResult;
}

export function demoIsEulerian(vcount: number, edges: Edge[]): AlgoResult {
  if (vcount === 0) return { has_path: false, has_cycle: false } as AlgoResult;
  const adj = buildAdj(vcount, edges);
  const visited = new Set<number>();
  let startNode = -1;
  for (let i = 0; i < vcount; i++) {
    if (adj[i]!.length > 0) { startNode = i; break; }
  }
  if (startNode < 0) return { has_path: edges.length === 0, has_cycle: edges.length === 0 } as AlgoResult;
  visited.add(startNode);
  const queue = [startNode];
  while (queue.length > 0) {
    const v = queue.shift()!;
    for (const w of adj[v]!) {
      if (!visited.has(w)) { visited.add(w); queue.push(w); }
    }
  }
  for (let i = 0; i < vcount; i++) {
    if (adj[i]!.length > 0 && !visited.has(i)) {
      return { has_path: false, has_cycle: false } as AlgoResult;
    }
  }
  let oddCount = 0;
  for (let i = 0; i < vcount; i++) {
    if (adj[i]!.length % 2 !== 0) oddCount++;
  }
  return {
    has_path: oddCount === 0 || oddCount === 2,
    has_cycle: oddCount === 0,
  } as AlgoResult;
}

export function demoCohesiveBlocks(vcount: number, edges: Edge[]): AlgoResult {
  const adj = buildAdj(vcount, edges);
  const allVerts = Array.from({ length: vcount }, (_, i) => i);
  const blocks: number[][] = [allVerts];
  const cohesion: number[] = [];

  function connectivity(subset: number[]): number {
    if (subset.length <= 1) return 0;
    const sset = new Set(subset);
    let minCut = subset.length;
    for (const r of subset) {
      const reached = new Set<number>();
      const start = subset.find(x => x !== r);
      if (start === undefined) return 0;
      reached.add(r);
      reached.add(start);
      const q = [start];
      while (q.length > 0) {
        const v = q.shift()!;
        for (const w of adj[v]!) {
          if (sset.has(w) && !reached.has(w)) { reached.add(w); q.push(w); }
        }
      }
      const reachCount = reached.size - 1;
      if (reachCount < subset.length - 1) {
        let cutSize = 0;
        for (const v of subset) {
          if (v !== r) {
            let hasUnreached = false;
            for (const w of adj[v]!) {
              if (sset.has(w) && !reached.has(w)) { hasUnreached = true; break; }
            }
            if (hasUnreached || !reached.has(v)) cutSize++;
          }
        }
        if (cutSize < minCut) minCut = cutSize;
      }
    }
    return Math.min(minCut, subset.length - 1);
  }

  for (const block of blocks) {
    cohesion.push(connectivity(block));
  }

  return { blocks, cohesion, count: blocks.length } as AlgoResult;
}

export function demoAvgNearestNeighborDegree(vcount: number, edges: Edge[]): AlgoResult {
  const adj = buildAdj(vcount, edges);
  const deg = new Array(vcount).fill(0);
  for (const [u, v] of edges) {
    if (u < vcount) deg[u]++;
    if (v < vcount) deg[v]++;
  }
  const scores: (number | null)[] = [];
  for (let v = 0; v < vcount; v++) {
    const nbrs = adj[v]!;
    if (nbrs.length === 0) { scores.push(null); continue; }
    let sum = 0;
    for (const u of nbrs) sum += deg[u]!;
    scores.push(sum / nbrs.length);
  }
  return { scores } as AlgoResult;
}

export function demoChromaticNumber(vcount: number, edges: Edge[]): AlgoResult {
  const result = demoColoring(vcount, edges) as { colors: number[]; chromatic: number };
  return { value: result.chromatic } as AlgoResult;
}

export function demoConvergenceDegree(vcount: number, edges: Edge[]): AlgoResult {
  const adj: Set<number>[] = Array.from({ length: vcount }, () => new Set());
  for (const [u, v] of edges) { adj[u]!.add(v); adj[v]!.add(u); }
  const scores: number[] = [];
  for (const [u, v] of edges) {
    let common = 0;
    for (const w of adj[u]!) {
      if (adj[v]!.has(w)) common++;
    }
    const unionSize = adj[u]!.size + adj[v]!.size - common;
    scores.push(unionSize > 0 ? common / unionSize : 0);
  }
  return { scores } as AlgoResult;
}

export function demoSimilarityJaccard(vcount: number, edges: Edge[]): AlgoResult {
  const adj: Set<number>[] = Array.from({ length: vcount }, () => new Set());
  for (const [u, v] of edges) { adj[u]!.add(v); adj[v]!.add(u); }
  const matrix: number[][] = [];
  for (let i = 0; i < vcount; i++) {
    const row: number[] = [];
    for (let j = 0; j < vcount; j++) {
      if (i === j) { row.push(1); continue; }
      let inter = 0;
      for (const w of adj[i]!) { if (adj[j]!.has(w)) inter++; }
      const union = adj[i]!.size + adj[j]!.size - inter;
      row.push(union > 0 ? inter / union : 0);
    }
    matrix.push(row);
  }
  return { matrix, size: vcount } as AlgoResult;
}

export function demoCommunityVoronoi(vcount: number, edges: Edge[]): AlgoResult {
  const result = demoComponents(vcount, edges) as { membership: number[]; count: number };
  return { membership: result.membership, generators: [], modularity: 0 } as AlgoResult;
}

export function demoGraphCenter(vcount: number, edges: Edge[]): AlgoResult {
  const ecc = demoEccentricity(vcount, edges) as { values: number[] };
  const minEcc = Math.min(...ecc.values);
  const vertices: number[] = [];
  ecc.values.forEach((e, i) => { if (e === minEcc) vertices.push(i); });
  return { vertices, count: vertices.length } as AlgoResult;
}

export function demoClusteringCoefficients(vcount: number, edges: Edge[]): AlgoResult {
  const adj = buildAdj(vcount, edges);
  const scores: (number | null)[] = [];
  for (let v = 0; v < vcount; v++) {
    const nbrs = adj[v]!;
    if (nbrs.length < 2) { scores.push(null); continue; }
    const nbrSet = new Set(nbrs);
    let triangles = 0;
    for (let i = 0; i < nbrs.length; i++) {
      for (let j = i + 1; j < nbrs.length; j++) {
        if (nbrSet.has(nbrs[j]!) && adj[nbrs[i]!]!.includes(nbrs[j]!)) triangles++;
      }
    }
    const pairs = nbrs.length * (nbrs.length - 1) / 2;
    scores.push(pairs > 0 ? triangles / pairs : 0);
  }
  return { scores } as AlgoResult;
}

export function demoAveragePathLength(vcount: number, edges: Edge[]): AlgoResult {
  if (vcount <= 1) return { value: 0 } as AlgoResult;
  const adj = buildAdj(vcount, edges);
  let totalDist = 0;
  let count = 0;
  for (let s = 0; s < vcount; s++) {
    const dist = new Int32Array(vcount).fill(-1);
    dist[s] = 0;
    const queue = [s];
    while (queue.length > 0) {
      const v = queue.shift()!;
      for (const w of adj[v]!) {
        if (dist[w]! < 0) {
          dist[w] = dist[v]! + 1;
          queue.push(w);
        }
      }
    }
    for (let t = s + 1; t < vcount; t++) {
      if (dist[t]! > 0) { totalDist += dist[t]!; count++; }
    }
  }
  return { value: count > 0 ? totalDist / count : 0 } as AlgoResult;
}

export function demoKShortestPaths(vcount: number, edges: Edge[], source = 0, target = 1, k = 5): AlgoResult {
  if (vcount === 0) return { paths: [], count: 0 } as AlgoResult;
  const adj = buildAdj(vcount, edges);
  const src = source >= 0 && source < vcount ? source : 0;
  const tgt = target >= 0 && target < vcount ? target : Math.min(1, vcount - 1);
  const paths: { vertices: number[]; weight: number }[] = [];
  const visited = new Set<number>();

  function dfs(u: number, path: number[]) {
    if (paths.length >= k) return;
    if (u === tgt) { paths.push({ vertices: [...path], weight: path.length - 1 }); return; }
    for (const w of adj[u]!) {
      if (!visited.has(w)) {
        visited.add(w);
        path.push(w);
        dfs(w, path);
        path.pop();
        visited.delete(w);
      }
    }
  }

  visited.add(src);
  dfs(src, [src]);
  paths.sort((a, b) => a.weight - b.weight);
  return { paths: paths.slice(0, k), count: Math.min(paths.length, k) } as AlgoResult;
}

export function runDemoAlgo(
  algo: string,
  vcount: number,
  edges: Edge[],
  params?: { damping?: number; source?: number; target?: number },
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
    case 'label_propagation':
      return demoLabelPropagation(vcount, edges);
    case 'walktrap':
      return demoWalktrap(vcount, edges);
    case 'leiden':
      return demoLeiden(vcount, edges);
    case 'fast_greedy':
      return demoFastGreedy(vcount, edges);
    case 'leading_eigenvector':
      return demoLeadingEigenvector(vcount, edges);
    case 'edge_betweenness':
      return demoEdgeBetweennessCommunity(vcount, edges);
    case 'fluid':
      return demoFluid(vcount, edges);
    case 'harmonic':
      return demoHarmonic(vcount, edges);
    case 'hits':
      return demoHits(vcount, edges);
    case 'katz':
      return demoKatz(vcount, edges);
    case 'dijkstra':
      return demoDijkstra(vcount, edges, params?.source);
    case 'graph_stats':
      return demoGraphStats(vcount, edges);
    case 'max_flow':
      return demoMaxFlow(vcount, edges);
    case 'articulation_points':
      return demoArticulationPoints(vcount, edges);
    case 'degree_sequence':
      return demoDegreeSequence(vcount, edges);
    case 'scc':
      return demoScc(vcount, edges);
    case 'bridges':
      return demoBridges(vcount, edges);
    case 'coloring':
      return demoColoring(vcount, edges);
    case 'topological_sort':
      return demoTopologicalSort(vcount, edges);
    case 'transitivity':
      return demoTransitivity(vcount, edges);
    case 'edge_betweenness_centrality':
      return demoEdgeBetweennessCentrality(vcount, edges);
    case 'triad_census':
      return demoTriadCensus(vcount, edges);
    case 'canonical_permutation':
      return demoCanonicalPermutation(vcount, edges);
    case 'count_automorphisms':
      return demoCountAutomorphisms(vcount, edges);
    case 'isomorphism':
      return demoIsomorphism(vcount, edges);
    case 'coreness':
      return demoCoreness(vcount, edges);
    case 'eccentricity':
      return demoEccentricity(vcount, edges);
    case 'constraint':
      return demoConstraint(vcount, edges);
    case 'diameter':
      return demoDiameter(vcount, edges);
    case 'shortest_path':
      return demoShortestPath(vcount, edges, params?.source, params?.target);
    case 'random_walk':
      return demoRandomWalk(vcount, edges, params?.source);
    case 'fundamental_cycles':
      return demoFundamentalCycles(vcount, edges);
    case 'list_triangles':
      return demoListTriangles(vcount, edges);
    case 'girth':
      return demoGirth(vcount, edges);
    case 'trussness':
      return demoTrussness(vcount, edges);
    case 'automorphism_group':
      return demoAutomorphismGroup(vcount, edges);
    case 'clique_number':
      return demoCliqueNumber(vcount, edges);
    case 'independence_number':
      return demoIndependenceNumber(vcount, edges);
    case 'maximal_cliques':
      return demoMaximalCliques(vcount, edges);
    case 'vertex_connectivity':
      return demoVertexConnectivity(vcount, edges);
    case 'edge_connectivity':
      return demoEdgeConnectivity(vcount, edges);
    case 'minimum_spanning_tree':
      return demoMinimumSpanningTree(vcount, edges);
    case 'bellman_ford':
      return demoBellmanFord(vcount, edges, params?.source);
    case 'degree_distribution':
      return demoDegreeDistribution(vcount, edges);
    case 'feedback_arc_set':
      return demoFeedbackArcSet(vcount, edges);
    case 'minimum_cycle_basis':
      return demoMinimumCycleBasis(vcount, edges);
    case 'biconnected_components':
      return demoBiconnectedComponents(vcount, edges);
    case 'bipartite_check':
      return demoBipartiteCheck(vcount, edges);
    case 'maximum_cut':
      return demoMaximumCut(vcount, edges);
    case 'global_efficiency':
      return demoGlobalEfficiency(vcount, edges);
    case 'local_efficiency':
      return demoLocalEfficiency(vcount, edges);
    case 'degeneracy':
      return demoDegeneracy(vcount, edges);
    case 'all_simple_paths':
      return demoAllSimplePaths(vcount, edges, params?.source, params?.target);
    case 'find_cycle':
      return demoFindCycle(vcount, edges);
    case 'mincut_value':
      return demoMincutValue(vcount, edges);
    case 'vertex_disjoint_paths':
      return demoVertexDisjointPaths(vcount, edges, params?.source, params?.target);
    case 'edge_disjoint_paths':
      return demoEdgeDisjointPaths(vcount, edges, params?.source, params?.target);
    case 'is_eulerian':
      return demoIsEulerian(vcount, edges);
    case 'cohesive_blocks':
      return demoCohesiveBlocks(vcount, edges);
    case 'avg_nearest_neighbor_degree':
      return demoAvgNearestNeighborDegree(vcount, edges);
    case 'chromatic_number':
      return demoChromaticNumber(vcount, edges);
    case 'convergence_degree':
      return demoConvergenceDegree(vcount, edges);
    case 'similarity_jaccard':
      return demoSimilarityJaccard(vcount, edges);
    case 'community_voronoi':
      return demoCommunityVoronoi(vcount, edges);
    case 'graph_center':
      return demoGraphCenter(vcount, edges);
    case 'clustering_coefficients':
      return demoClusteringCoefficients(vcount, edges);
    case 'average_path_length':
      return demoAveragePathLength(vcount, edges);
    case 'k_shortest_paths':
      return demoKShortestPaths(vcount, edges, params?.source, params?.target);
    default:
      return demoPagerank(vcount, edges);
  }
}
