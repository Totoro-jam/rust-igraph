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

This package is published via [npm Trusted Publishing (OIDC)](https://docs.npmjs.com/generating-provenance-statements)
— zero static tokens, cryptographically signed provenance attestation.

Release workflow: push a `wasm-v*` tag (e.g. `wasm-v0.2.0`) to trigger
`.github/workflows/release-wasm.yml`.

## License

GPL-2.0-or-later — compiled from [rust-igraph](https://github.com/Totoro-jam/rust-igraph).

The higher-level TypeScript wrapper packages (`@graphrs/*`) are MIT licensed.

---

# @graphrs/igraph-wasm

纯 Rust 实现的 [igraph](https://igraph.org)，编译为 WebAssembly。提供 400+
图算法（社区发现、中心性、布局、流、同构等），在浏览器和 Node.js 中以接近原生的速度运行。

## 使用方式

这是**底层 WASM 二进制包**。大多数用户应安装
[`@graphrs/core`](https://www.npmjs.com/package/@graphrs/core) 获得更友好的
TypeScript API。

```js
import init, { WasmGraph } from "@graphrs/igraph-wasm";

await init();
const g = WasmGraph.fromEdges(new Uint32Array([0,1, 1,2, 2,0]), false);
const result = JSON.parse(g.louvain());
console.log(result.membership); // [0, 0, 0]
```

## 环境支持

| 环境 | 要求 |
|------|------|
| 浏览器 | Chrome 89+, Firefox 89+, Safari 15+, Edge 89+ |
| Node.js | 18+（ESM，支持 top-level await） |
| 打包工具 | Vite 5+, webpack 5+, Rollup 4+, esbuild |

## 包体积

| 文件 | 原始大小 | Gzip 后 |
|------|----------|---------|
| `.wasm` | ~2.4 MB | ~800 KB |
| `.js`（胶水代码） | ~215 KB | ~45 KB |

## 发布方式

本包通过 [npm Trusted Publishing (OIDC)](https://docs.npmjs.com/generating-provenance-statements)
发布——零静态令牌，带密码学签名的溯源证明。

发布流程：推送 `wasm-v*` 标签（如 `wasm-v0.2.0`）触发
`.github/workflows/release-wasm.yml`。

## 许可证

GPL-2.0-or-later — 编译自 [rust-igraph](https://github.com/Totoro-jam/rust-igraph)。

上层 TypeScript 封装包（`@graphrs/*`）采用 MIT 许可证。
