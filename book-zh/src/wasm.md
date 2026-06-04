# 在浏览器中使用 rust-igraph (WASM)

rust-igraph 可编译为 WebAssembly，让你在浏览器中直接运行图算法，
无需后端服务器。本章将完整介绍从构建 WASM 模块到集成进 Web 应用的
全部流程。

## 前置要求

你需要：

- **Rust** (1.85+) 并安装 `wasm32-unknown-unknown` 编译目标
- **wasm-pack**（封装 `wasm-bindgen` 的构建工具）
- **Node.js** (18+) 用于前端工具链

```bash
# 安装 WASM 编译目标
rustup target add wasm32-unknown-unknown

# 安装 wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
# 或通过 cargo 安装：
cargo install wasm-pack
```

## 构建 WASM 模块

`igraph-wasm` crate 位于 `crates/igraph-wasm/`，提供浏览器端绑定。
使用 wasm-pack 构建：

```bash
# 克隆仓库（如尚未克隆）
git clone https://github.com/Totoro-jam/rust-igraph
cd rust-igraph

# 构建浏览器版（ES module 输出）
wasm-pack build crates/igraph-wasm \
  --target web \
  --out-dir ../../pkg \
  --out-name igraph_wasm
```

构建完成后会在 `pkg/` 目录生成以下文件：

| 文件 | 用途 |
|------|------|
| `igraph_wasm.js` | JavaScript 胶水代码（ES 模块） |
| `igraph_wasm_bg.wasm` | 编译后的 WebAssembly 二进制 |
| `igraph_wasm.d.ts` | TypeScript 类型定义 |
| `package.json` | npm 包元数据 |

### 构建目标

wasm-pack 支持不同的输出目标：

```bash
# ES 模块（Vite、webpack、带 <script type="module"> 的浏览器）
wasm-pack build crates/igraph-wasm --target web

# 打包器用（webpack、Rollup — 使用 import()）
wasm-pack build crates/igraph-wasm --target bundler

# Node.js 用（CommonJS）
wasm-pack build crates/igraph-wasm --target nodejs
```

大多数 Web 项目应选择 `--target web`。

## 快速开始 — 原生 HTML

最简单的方式是直接在 `<script>` 标签中使用：

```html
<!DOCTYPE html>
<html>
<head>
  <title>rust-igraph WASM 演示</title>
</head>
<body>
  <pre id="output"></pre>
  <script type="module">
    // 导入并初始化 WASM 模块
    import init, { WasmGraph } from './pkg/igraph_wasm.js';

    async function main() {
      // 使用任何 WASM 函数前必须先调用 init()
      await init();

      // 通过边对创建图：[源0, 目标0, 源1, 目标1, ...]
      const edges = new Uint32Array([0,1, 1,2, 2,3, 3,0, 0,2, 1,3]);
      const graph = WasmGraph.fromEdges(edges, false); // 无向图

      console.log(`顶点数: ${graph.vcount()}, 边数: ${graph.ecount()}`);

      // 运行 PageRank
      const prJson = graph.pagerank();
      const pr = JSON.parse(prJson);
      document.getElementById('output').textContent =
        `PageRank 分数: ${pr.scores.map(s => s.toFixed(4)).join(', ')}`;

      // 运行社区检测
      const commJson = graph.louvain();
      const comm = JSON.parse(commJson);
      console.log('社区:', comm.membership);
      console.log('模块度:', comm.modularity.toFixed(4));

      // 用完后务必释放图
      graph.free();
    }

    main();
  </script>
</body>
</html>
```

使用任意静态文件服务器提供服务（如 `python3 -m http.server`），
然后在浏览器中打开即可。

## 与 Vite + React 集成

生产级别的 Vite + React 配置：

### 1. 将 WASM 模块构建到项目中

```bash
wasm-pack build crates/igraph-wasm \
  --target web \
  --out-dir ../../my-app/public/wasm \
  --out-name igraph_wasm
```

将输出放在 `public/wasm/` 下，确保 Vite 原样复制文件而不添加哈希
（WASM 文件需要稳定路径以支持动态导入）。

### 2. 创建 Web Worker

在 Web Worker 中运行图算法可保持 UI 响应。创建 `src/worker.ts`：

```typescript
// 匹配 WASM 导出的 TypeScript 接口
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
    console.error('WASM 初始化失败:', e);
    return false;
  }
}

// 消息处理
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
      const resultJson = graph.pagerank(); // 或其他算法
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

### 3. 在 React 中使用 Worker

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
        运行 PageRank
      </button>
      {result && <pre>{JSON.stringify(result, null, 2)}</pre>}
    </div>
  );
}
```

## 与 Node.js 集成

服务端或 CLI 场景下使用 Node.js：

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

## WasmGraph API 参考

所有算法方法返回 JSON 字符串。使用 `JSON.parse()` 解析后即可
获取下述结果对象。

### 构造方法

| 方法 | 说明 |
|------|------|
| `WasmGraph.fromEdges(edges: Uint32Array, directed: boolean)` | 从扁平边对 `[u0,v0, u1,v1, ...]` 创建 |
| `new WasmGraph(directed: boolean)` | 创建空图 |
| `graph.addEdge(u: number, v: number)` | 添加单条边 |

### 属性方法

| 方法 | 返回类型 | 说明 |
|------|----------|------|
| `graph.vcount()` | `number` | 顶点数 |
| `graph.ecount()` | `number` | 边数 |

### 中心性算法

| 方法 | 结果字段 | 说明 |
|------|----------|------|
| `graph.pagerank()` | `{ scores: number[] }` | PageRank 中心性 |
| `graph.betweenness()` | `{ scores: number[] }` | 介数中心性 |
| `graph.closeness()` | `{ scores: number[] }` | 接近中心性 |
| `graph.eigenvectorCentrality()` | `{ scores: number[] }` | 特征向量中心性 |
| `graph.harmonicCentrality()` | `{ scores: number[] }` | 调和中心性 |
| `graph.katzCentrality()` | `{ scores: number[] }` | Katz 中心性 |
| `graph.hubAndAuthorityScores()` | `{ hub: number[], authority: number[] }` | HITS 算法 |
| `graph.edgeBetweenness()` | `{ scores: number[] }` | 边介数中心性 |

### 社区检测

| 方法 | 结果字段 | 说明 |
|------|----------|------|
| `graph.louvain()` | `{ membership: number[], modularity: number }` | Louvain 模块度 |
| `graph.leiden()` | `{ membership: number[], quality: number, nb_clusters: number }` | Leiden 算法 |
| `graph.infomap()` | `{ membership: number[], codelength: number }` | Infomap |
| `graph.spinglass()` | `{ membership: number[], modularity: number, nb_clusters: number }` | Spinglass |
| `graph.labelPropagation()` | `{ membership: number[], nb_clusters: number }` | 标签传播 |
| `graph.walktrap()` | `{ membership: number[], nb_clusters: number, modularity: number }` | Walktrap 随机游走 |
| `graph.fastGreedy()` | `{ membership: number[], nb_clusters: number, modularity: number }` | 快速贪心 |
| `graph.leadingEigenvector()` | `{ membership: number[], modularity: number }` | 主特征向量法 |
| `graph.edgeBetweennessCommunity()` | `{ membership: number[], nb_clusters: number }` | 边介数社区检测 |
| `graph.fluidCommunities(k: number)` | `{ membership: number[], nb_clusters: number }` | 流体社区检测 |

### 遍历与路径

| 方法 | 结果字段 | 说明 |
|------|----------|------|
| `graph.bfs(root: number)` | `{ order: number[] }` | 广度优先搜索 |
| `graph.dfs(root: number)` | `{ order: number[] }` | 深度优先搜索 |
| `graph.dijkstra(source: number, weights: Float64Array)` | `{ distances: number[] }` | Dijkstra 最短路径 |
| `graph.topologicalSort()` | `{ order: number[] }` | 拓扑排序（有向图） |
| `graph.maxFlow(source: number, target: number)` | `{ value: number }` | 最大流 |

### 图结构

| 方法 | 结果字段 | 说明 |
|------|----------|------|
| `graph.connectedComponents()` | `{ membership: number[], count: number }` | 弱连通分量 |
| `graph.stronglyConnectedComponents()` | `{ membership: number[], count: number }` | 强连通分量 |
| `graph.graphStats()` | `{ vcount, ecount, is_directed, is_connected, diameter, girth, triangles, is_bipartite }` | 聚合统计 |
| `graph.articulationPoints()` | `{ vertices: number[] }` | 割点 |
| `graph.bridges()` | `{ edges: [number,number][], count: number }` | 桥边 |
| `graph.degreeSequence()` | `{ degrees: number[] }` | 各顶点度 |
| `graph.vertexColoring()` | `{ colors: number[], chromatic: number }` | 贪心顶点着色 |
| `graph.transitivity()` | `{ value: number }` | 全局聚类系数 |

### 布局

| 方法 | 结果字段 | 说明 |
|------|----------|------|
| `graph.layoutFr(niter: number)` | `{ coords: [number,number][] }` | Fruchterman-Reingold 力导向布局 |

### 内存管理

使用完图实例后务必调用 `graph.free()`。WASM 内存不受 JavaScript
垃圾回收管理 —— 不调用 `free()` 将导致内存泄漏，直到页面关闭或
Worker 终止。

```javascript
const graph = WasmGraph.fromEdges(edges, false);
try {
  const result = JSON.parse(graph.pagerank());
  // 使用 result...
} finally {
  graph.free(); // 务必释放！
}
```

## 性能建议

1. **使用 Web Workers** —— 图算法可能非常消耗 CPU。在主线程运行会
   阻塞 UI，应始终放到 Worker 中执行。

2. **复用图实例** —— 如需对同一张图运行多个算法，创建一次
   `WasmGraph`，调用多个方法后再释放。

   ```javascript
   const graph = WasmGraph.fromEdges(edges, false);
   const pr = JSON.parse(graph.pagerank());
   const bc = JSON.parse(graph.betweenness());
   const layout = JSON.parse(graph.layoutFr(300));
   graph.free();
   ```

3. **使用类型化数组** —— 边必须以 `Uint32Array` 传入，不能用普通
   数组。Dijkstra 权重使用 `Float64Array`。类型化数组避免了 JS/WASM
   边界的序列化开销。

4. **包体积** —— WASM 二进制 gzip 后约 400 KB。生产构建请使用
   `wasm-opt -O3`（`wasm-pack build --release` 默认包含）。

## 常见问题

### "WASM module not found" / 404 错误

WASM 文件必须以正确的 MIME 类型（`application/wasm`）提供服务。
大多数开发服务器会自动处理。如使用 Vite，将 WASM 输出放在
`public/` 目录下即可原样提供：

```bash
wasm-pack build crates/igraph-wasm \
  --target web \
  --out-dir ../../my-app/public/wasm
```

### "Cannot use import statement outside a module"

`--target web` 的输出是 ES 模块。请使用 `<script type="module">`
或配置打包器正确处理 `.js` 导入。

### 加载 WASM 时出现 CORS 错误

从 `file://` 加载时，CORS 策略会阻止 WASM。请使用本地服务器：

```bash
python3 -m http.server 8080
# 或
npx serve .
```

### 大图的内存问题

WASM 有默认内存上限。对于数百万条边的图，可能需要在
`.cargo/config.toml` 中增加：

```toml
[target.wasm32-unknown-unknown]
rustflags = ["-C", "link-args=-z stack-size=8388608"]
```

### 要求无向图的算法

部分算法（spinglass、fast greedy、fluid communities）要求无向图。
传入有向图时，WASM 调用会抛出带描述信息的 JavaScript `Error`。

## 在线演示

试用[交互式实验场](https://totoro-jam.github.io/rust-igraph/playground/)，
可在浏览器中通过 WASM 运行 30+ 种算法。实验场源代码位于
`website/playground/`，是完整的参考实现。

## 下一步

- 浏览 [API 文档](./api.md) 了解完整的 Rust API
- 查看[实验场源码](https://github.com/Totoro-jam/rust-igraph/tree/main/website/playground)
  获取生产级 React + WASM 集成示例
- 运行 `cargo check --target wasm32-unknown-unknown` 验证你的代码
  是否兼容 WASM
