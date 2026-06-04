# Using rust-igraph in the Browser (WASM)

rust-igraph compiles to WebAssembly, letting you run graph algorithms
entirely client-side — no server needed. This chapter walks through
the full path from building the WASM module to integrating it into
your web application.

## Prerequisites

You need:

- **Rust** (1.85+) with the `wasm32-unknown-unknown` target
- **wasm-pack** (the build tool that wraps `wasm-bindgen`)
- **Node.js** (18+) for your frontend toolchain

```bash
# Install the WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
# or via cargo:
cargo install wasm-pack
```

## Building the WASM module

The `igraph-wasm` crate in `crates/igraph-wasm/` provides the browser
bindings. Build it with wasm-pack:

```bash
# Clone the repo (if you haven't already)
git clone https://github.com/Totoro-jam/rust-igraph
cd rust-igraph

# Build for browser (ES module output)
wasm-pack build crates/igraph-wasm \
  --target web \
  --out-dir ../../pkg \
  --out-name igraph_wasm
```

This produces a `pkg/` directory containing:

| File | Purpose |
|------|---------|
| `igraph_wasm.js` | JavaScript glue (ES module) |
| `igraph_wasm_bg.wasm` | Compiled WebAssembly binary |
| `igraph_wasm.d.ts` | TypeScript type definitions |
| `package.json` | npm package metadata |

### Build targets

wasm-pack supports different output targets:

```bash
# For ES module usage (Vite, webpack, browsers with <script type="module">)
wasm-pack build crates/igraph-wasm --target web

# For bundlers (webpack, Rollup — uses import())
wasm-pack build crates/igraph-wasm --target bundler

# For Node.js (CommonJS)
wasm-pack build crates/igraph-wasm --target nodejs
```

For most web projects, `--target web` is the right choice.

## Quick start — vanilla HTML

The simplest way to use the WASM module is directly in a `<script>` tag:

```html
<!DOCTYPE html>
<html>
<head>
  <title>rust-igraph WASM Demo</title>
</head>
<body>
  <pre id="output"></pre>
  <script type="module">
    // Import and initialize the WASM module
    import init, { WasmGraph } from './pkg/igraph_wasm.js';

    async function main() {
      // Must call init() before using any WASM functions
      await init();

      // Create a graph from edge pairs: [src0, dst0, src1, dst1, ...]
      const edges = new Uint32Array([0,1, 1,2, 2,3, 3,0, 0,2, 1,3]);
      const graph = WasmGraph.fromEdges(edges, false); // undirected

      console.log(`Vertices: ${graph.vcount()}, Edges: ${graph.ecount()}`);

      // Run PageRank
      const prJson = graph.pagerank();
      const pr = JSON.parse(prJson);
      document.getElementById('output').textContent =
        `PageRank scores: ${pr.scores.map(s => s.toFixed(4)).join(', ')}`;

      // Run community detection
      const commJson = graph.louvain();
      const comm = JSON.parse(commJson);
      console.log('Communities:', comm.membership);
      console.log('Modularity:', comm.modularity.toFixed(4));

      // Always free the graph when done
      graph.free();
    }

    main();
  </script>
</body>
</html>
```

Serve this with any static file server (e.g. `python3 -m http.server`)
and open it in your browser.

## Integration with Vite + React

For a production-quality setup with Vite and React:

### 1. Build the WASM module into your project

```bash
wasm-pack build crates/igraph-wasm \
  --target web \
  --out-dir ../../my-app/public/wasm \
  --out-name igraph_wasm
```

Placing the output in `public/wasm/` ensures Vite copies the files as-is
without hashing (WASM files need stable paths for dynamic import).

### 2. Create a Web Worker

Running graph algorithms in a Web Worker keeps the UI responsive. Create
`src/worker.ts`:

```typescript
// TypeScript interface matching the WASM exports
interface WasmGraphInstance {
  bfs(root: number): string;
  dfs(root: number): string;
  pagerank(): string;
  louvain(): string;
  betweenness(): string;
  closeness(): string;
  connectedComponents(): string;
  layoutFr(niter: number): string;
  vcount(): number;
  ecount(): number;
  free(): void;
}

let WasmGraph: {
  fromEdges(edges: Uint32Array, directed: boolean): WasmGraphInstance;
} | null = null;

async function initWasm(): Promise<boolean> {
  try {
    const workerUrl = self.location.href;
    const root = workerUrl.replace(/\/[^/]*$/, '');
    const wasmModule = await import(
      /* @vite-ignore */ `${root}/wasm/igraph_wasm.js`
    );
    await wasmModule.default();
    WasmGraph = wasmModule.WasmGraph;
    return true;
  } catch (e) {
    console.error('WASM init failed:', e);
    return false;
  }
}

// Message handler
self.onmessage = async (e: MessageEvent) => {
  const { type, ...params } = e.data;

  if (type === 'init') {
    const ok = await initWasm();
    self.postMessage({ type: 'ready', ok });
    return;
  }

  if (type === 'run' && WasmGraph) {
    const edges = new Uint32Array(params.edges);
    const graph = WasmGraph.fromEdges(edges, params.directed);

    try {
      const t0 = performance.now();
      const resultJson = graph.pagerank(); // or any algorithm
      const elapsed = performance.now() - t0;

      self.postMessage({
        type: 'result',
        data: JSON.parse(resultJson),
        elapsed,
      });
    } finally {
      graph.free();
    }
  }
};
```

### 3. Use the worker from React

```tsx
import { useEffect, useRef, useState } from 'react';

function App() {
  const workerRef = useRef<Worker | null>(null);
  const [ready, setReady] = useState(false);
  const [result, setResult] = useState<any>(null);

  useEffect(() => {
    const worker = new Worker(
      new URL('./worker.ts', import.meta.url),
      { type: 'module' }
    );

    worker.onmessage = (e) => {
      if (e.data.type === 'ready') setReady(e.data.ok);
      if (e.data.type === 'result') setResult(e.data.data);
    };

    worker.postMessage({ type: 'init' });
    workerRef.current = worker;

    return () => worker.terminate();
  }, []);

  const runPageRank = () => {
    workerRef.current?.postMessage({
      type: 'run',
      edges: [0,1, 1,2, 2,3, 3,0, 0,2],
      directed: false,
    });
  };

  return (
    <div>
      <button onClick={runPageRank} disabled={!ready}>
        Run PageRank
      </button>
      {result && <pre>{JSON.stringify(result, null, 2)}</pre>}
    </div>
  );
}
```

## Integration with Node.js

For server-side or CLI usage with Node.js:

```bash
wasm-pack build crates/igraph-wasm --target nodejs --out-dir ../../pkg
```

```javascript
const { WasmGraph } = require('./pkg/igraph_wasm.js');

const edges = new Uint32Array([0,1, 1,2, 2,0, 2,3, 3,4]);
const graph = WasmGraph.fromEdges(edges, false);

const pr = JSON.parse(graph.pagerank());
console.log('PageRank:', pr.scores);

graph.free();
```

## WasmGraph API reference

All algorithm methods return JSON strings. Parse them with
`JSON.parse()` to get the result objects described below.

### Construction

| Method | Description |
|--------|-------------|
| `WasmGraph.fromEdges(edges: Uint32Array, directed: boolean)` | Create from flat edge pairs `[u0,v0, u1,v1, ...]` |
| `new WasmGraph(directed: boolean)` | Create an empty graph |
| `graph.addEdge(u: number, v: number)` | Add a single edge |

### Graph generators (static constructors)

| Method | Description |
|--------|-------------|
| `WasmGraph.erdosRenyi(n, p, seed)` | Erdos-Renyi G(n,p) random graph |
| `WasmGraph.fullGraph(n)` | Complete graph K_n |
| `WasmGraph.cycleGraph(n)` | Cycle graph C_n |
| `WasmGraph.ringGraph(n, circular)` | Ring (cycle if circular=true, path if false) |
| `WasmGraph.barabasiAlbert(n, m, seed)` | Barabasi-Albert preferential attachment |
| `WasmGraph.wattsStrogatz(n, k, p, seed)` | Watts-Strogatz small-world |

### Properties

| Method | Return type | Description |
|--------|-------------|-------------|
| `graph.vcount()` | `number` | Number of vertices |
| `graph.ecount()` | `number` | Number of edges |

### Centrality algorithms

| Method | Result fields | Description |
|--------|---------------|-------------|
| `graph.pagerank()` | `{ scores: number[] }` | PageRank centrality |
| `graph.betweenness()` | `{ scores: number[] }` | Betweenness centrality |
| `graph.closeness()` | `{ scores: number[] }` | Closeness centrality |
| `graph.eigenvectorCentrality()` | `{ scores: number[] }` | Eigenvector centrality |
| `graph.harmonicCentrality()` | `{ scores: number[] }` | Harmonic centrality |
| `graph.katzCentrality()` | `{ scores: number[] }` | Katz centrality |
| `graph.hubAndAuthorityScores()` | `{ hub: number[], authority: number[] }` | HITS algorithm |
| `graph.edgeBetweenness()` | `{ scores: number[] }` | Edge betweenness centrality |

### Community detection

| Method | Result fields | Description |
|--------|---------------|-------------|
| `graph.louvain()` | `{ membership: number[], modularity: number }` | Louvain modularity |
| `graph.leiden()` | `{ membership: number[], quality: number, nb_clusters: number }` | Leiden algorithm |
| `graph.infomap()` | `{ membership: number[], codelength: number }` | Infomap |
| `graph.spinglass()` | `{ membership: number[], modularity: number, nb_clusters: number }` | Spinglass |
| `graph.labelPropagation()` | `{ membership: number[], nb_clusters: number }` | Label propagation |
| `graph.walktrap()` | `{ membership: number[], nb_clusters: number, modularity: number }` | Walktrap |
| `graph.fastGreedy()` | `{ membership: number[], nb_clusters: number, modularity: number }` | Fast greedy |
| `graph.leadingEigenvector()` | `{ membership: number[], modularity: number }` | Leading eigenvector |
| `graph.edgeBetweennessCommunity()` | `{ membership: number[], nb_clusters: number }` | Edge betweenness community |
| `graph.fluidCommunities(k: number)` | `{ membership: number[], nb_clusters: number }` | Fluid communities |

### Traversal & paths

| Method | Result fields | Description |
|--------|---------------|-------------|
| `graph.bfs(root: number)` | `{ order: number[] }` | Breadth-first search |
| `graph.dfs(root: number)` | `{ order: number[] }` | Depth-first search |
| `graph.dijkstra(source: number, weights: Float64Array)` | `{ distances: number[] }` | Dijkstra shortest paths |
| `graph.shortestPath(source, target)` | `{ path: number[] }` | Shortest path between two vertices |
| `graph.randomWalk(start, steps, seed)` | `{ vertices: number[] }` | Random walk from a start vertex |
| `graph.topologicalSort()` | `{ order: number[] }` | Topological sort (directed graphs) |
| `graph.maxFlow(source: number, target: number)` | `{ value: number }` | Maximum flow |
| `graph.diameter()` | `{ diameter: number \| null }` | Graph diameter (longest shortest path) |

### Structure

| Method | Result fields | Description |
|--------|---------------|-------------|
| `graph.connectedComponents()` | `{ membership: number[], count: number }` | Weakly connected components |
| `graph.stronglyConnectedComponents()` | `{ membership: number[], count: number }` | Strongly connected components |
| `graph.graphStats()` | `{ vcount, ecount, is_directed, is_connected, diameter, girth, triangles, is_bipartite }` | Aggregate statistics |
| `graph.articulationPoints()` | `{ vertices: number[] }` | Cut vertices |
| `graph.bridges()` | `{ edges: [number,number][], count: number }` | Bridge edges |
| `graph.degreeSequence()` | `{ degrees: number[] }` | Degree of each vertex |
| `graph.vertexColoring()` | `{ colors: number[], chromatic: number }` | Greedy vertex coloring |
| `graph.transitivity()` | `{ value: number }` | Global clustering coefficient |
| `graph.triadCensus()` | `{ counts: number[] }` | 16-type triad census (directed) |

### Isomorphism

| Method | Result fields | Description |
|--------|---------------|-------------|
| `graph.canonicalPermutation()` | `{ permutation: number[] }` | BLISS canonical labeling |
| `graph.countAutomorphisms()` | `{ count: number }` | Automorphism group order |
| `graph.isomorphicBliss(other)` | `{ isomorphic: boolean, mapping: number[] }` | BLISS isomorphism test |

### Layout

| Method | Result fields | Description |
|--------|---------------|-------------|
| `graph.layoutFr(niter: number)` | `{ coords: [number,number][] }` | Fruchterman-Reingold force-directed layout |

### Memory management

Always call `graph.free()` when you're done with a graph instance.
WASM memory is not garbage-collected by JavaScript — if you don't call
`free()`, the graph's memory will leak until the page is closed or the
worker is terminated.

```javascript
const graph = WasmGraph.fromEdges(edges, false);
try {
  const result = JSON.parse(graph.pagerank());
  // use result...
} finally {
  graph.free(); // always free!
}
```

## Performance tips

1. **Use Web Workers** — graph algorithms can be CPU-intensive. Running
   them on the main thread blocks the UI. Always offload to a worker.

2. **Reuse graph instances** — if you run multiple algorithms on the same
   graph, create the `WasmGraph` once and call multiple methods before
   freeing.

   ```javascript
   const graph = WasmGraph.fromEdges(edges, false);
   const pr = JSON.parse(graph.pagerank());
   const bc = JSON.parse(graph.betweenness());
   const layout = JSON.parse(graph.layoutFr(300));
   graph.free();
   ```

3. **Use typed arrays** — edges must be passed as `Uint32Array`, not
   regular arrays. Dijkstra weights use `Float64Array`. Typed arrays
   avoid serialization overhead across the JS/WASM boundary.

4. **Bundle size** — the WASM binary is ~400 KB gzipped. Use
   `wasm-opt -O3` (included in `wasm-pack build --release`) for
   production builds.

## Troubleshooting

### "WASM module not found" / 404 errors

The WASM files must be served with the correct MIME type
(`application/wasm`). Most development servers handle this automatically.
If you're using Vite, place the WASM output in `public/` so it's served
as-is:

```bash
wasm-pack build crates/igraph-wasm \
  --target web \
  --out-dir ../../my-app/public/wasm
```

### "Cannot use import statement outside a module"

The `--target web` output is an ES module. Use `<script type="module">`
or configure your bundler to handle `.js` imports correctly.

### CORS errors when loading WASM

If loading from `file://`, CORS policies block WASM. Use a local server:

```bash
python3 -m http.server 8080
# or
npx serve .
```

### Memory issues with large graphs

WASM has a default memory limit. For graphs with millions of edges,
you may need to increase it in `.cargo/config.toml`:

```toml
[target.wasm32-unknown-unknown]
rustflags = ["-C", "link-args=-z stack-size=8388608"]
```

### Algorithms that require undirected graphs

Some algorithms (spinglass, fast greedy, fluid communities) require
undirected graphs. If you pass a directed graph, the WASM call will
throw a JavaScript `Error` with a descriptive message.

## Live demo

Try the [interactive playground](https://totoro-jam.github.io/rust-igraph/playground/)
to see all 50 algorithms running in the browser via WASM. The
playground source code at `website/playground/` serves as a full
reference implementation.

## Next steps

- Browse the [API documentation](./api.md) for the full Rust API
- Check the [playground source](https://github.com/Totoro-jam/rust-igraph/tree/main/website/playground)
  for a production-quality React + WASM integration
- Run `cargo check --target wasm32-unknown-unknown` to verify your own
  code is WASM-compatible
