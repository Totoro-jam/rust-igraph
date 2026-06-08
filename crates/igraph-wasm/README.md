[English](README.md) | [中文](README.zh-CN.md)

# @graphrs/igraph-wasm

Pure-Rust [igraph](https://igraph.org) compiled to WebAssembly. Provides 400+
graph algorithms (community detection, centrality, layout, flow, isomorphism,
and more) at native speed in the browser and Node.js.

## Usage

This is the **low-level WASM binary**. Most users should install
[`@graphrs/core`](https://www.npmjs.com/package/@graphrs/core) for an ergonomic
TypeScript API.

```js
import init, { WasmGraph } from "@graphrs/igraph-wasm";

await init();
const g = WasmGraph.fromEdges(new Uint32Array([0,1, 1,2, 2,0]), false);
const result = JSON.parse(g.louvain());
console.log(result.membership); // [0, 0, 0]
```

## Environment Support

| Environment | Requirement |
|-------------|-------------|
| Browser | Chrome 89+, Firefox 89+, Safari 15+, Edge 89+ |
| Node.js | 18+ (ESM with top-level await) |
| Bundlers | Vite 5+, webpack 5+, Rollup 4+, esbuild |

## Bundle Size

| File | Raw | Gzipped |
|------|-----|---------|
| `.wasm` | ~2.4 MB | ~800 KB |
| `.js` (glue) | ~215 KB | ~45 KB |

## Publishing

Published via [npm Trusted Publishing (OIDC)](https://docs.npmjs.com/generating-provenance-statements)
— zero static tokens, cryptographically signed provenance attestation.

Release: push a `wasm-v*` tag (e.g. `wasm-v0.2.0`) to trigger
`.github/workflows/release-wasm.yml`.

## License

GPL-2.0-or-later — compiled from [rust-igraph](https://github.com/Totoro-jam/rust-igraph).

The higher-level TypeScript wrapper packages (`@graphrs/*`) are MIT licensed.
