[English](README.md) | [中文](README.zh-CN.md)

# rust-igraph

[![crates.io](https://img.shields.io/crates/v/rust-igraph.svg?label=crates.io)](https://crates.io/crates/rust-igraph)
[![docs.rs](https://img.shields.io/docsrs/rust-igraph?label=docs.rs)](https://docs.rs/rust-igraph)
[![CI](https://github.com/Totoro-jam/rust-igraph/actions/workflows/ci.yml/badge.svg)](https://github.com/Totoro-jam/rust-igraph/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Totoro-jam/rust-igraph/branch/main/graph/badge.svg)](https://codecov.io/gh/Totoro-jam/rust-igraph)
[![License: GPL-2.0-or-later](https://img.shields.io/badge/license-GPL--2.0--or--later-blue.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-orange.svg)](Cargo.toml)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

**纯 Rust 高性能图与网络分析库。** 忠实移植自 [igraph](https://igraph.org/)，提供 1,297 个公开 API，
零 `unsafe`，无 C/C++ FFI。

为研究人员、数据科学家和系统工程师打造，无需离开 Rust 生态即可获得生产级图算法。

## 为什么选择 rust-igraph？

| | rust-igraph | petgraph | igraph (C/Python) |
|---|---|---|---|
| **算法覆盖** | 1,297 个 API（BFS、DFS、最短路径、社区发现、中心性、同构、流、布局、图生成器、60+ 图结构识别器…） | ~50（可组合） | ~850 个 API（参考实现） |
| **安全性** | 零 `unsafe`，库代码中零 `unwrap` | 少量 `unsafe` | C 核心 + 绑定 |
| **正确性** | 与 igraph C、python-igraph、R-igraph 测试套件交叉验证 | 独立实现 | 参考实现 |
| **依赖** | 极简（1 个运行时依赖：`thiserror`） | 极简 | 需要 C/C++ 工具链 |
| **WASM** | 原生支持 `wasm32-unknown-unknown` | 支持 | 不支持 |

## 功能特性

- **遍历**: BFS、DFS、拓扑排序、随机游走
- **最短路径**: Dijkstra、Bellman-Ford、A*、全源最短路径、最宽路径
- **中心性**: betweenness、closeness、特征向量中心性、PageRank、HITS、Katz、harmonic、constraint
- **社区发现**: Louvain、Leiden、Infomap、Spinglass、标签传播、Walktrap、边 betweenness、fast greedy、leading eigenvector、fluid communities、Voronoi
- **连通性**: 连通/双连通分量、关节点、桥、分离集、凝聚块、强连通分量
- **网络流**: 最大流（push-relabel）、最小割、Gomory-Hu 树、边/顶点连通度、不相交路径
- **同构**: VF2（图/子图）、LAD 子图、BLISS 规范标记、自同构群
- **图生成器**: Erdos-Renyi、Barabasi-Albert、Watts-Strogatz、SBM、forest fire、geometric random、度序列、格子图、经典图，共 30+ 种
- **图属性**: 60+ 结构识别器（`is_bipartite`、`is_chordal`、`is_planar`、`is_perfect`、`is_cograph`、`is_series_parallel`…）
- **特征值求解器**: Lanczos（对称）、Arnoldi（一般）、图邻接矩阵
- **布局**: Fruchterman-Reingold、Kamada-Kawai、DrL、Sugiyama、GEM、Davidson-Harel、GraphOpt、MDS、LGL、UMAP、Reingold-Tilford、circle、star、grid、bipartite（16 种引擎，2D+3D）
- **空间算法**: Delaunay 三角剖分、Gabriel 图、beta-skeleton、最近邻图
- **I/O**: GML、GraphML、Pajek、DOT/Graphviz、LEDA、UCINET DL、DIMACS、边列表、NCOL、LGL、GraphDB（15 种读写函数）

## 快速开始

添加到 `Cargo.toml`：

```toml
[dependencies]
rust-igraph = "0.7"
```

```rust
use rust_igraph::{Graph, bfs};

fn main() {
    // 构建一个小型社交网络
    let mut g = Graph::with_vertices(6);
    g.add_edge(0, 1).unwrap(); // Alice - Bob
    g.add_edge(0, 2).unwrap(); // Alice - Carol
    g.add_edge(1, 3).unwrap(); // Bob - Dave
    g.add_edge(2, 4).unwrap(); // Carol - Eve
    g.add_edge(3, 5).unwrap(); // Dave - Frank

    // 从 Alice 开始 BFS
    let order = bfs(&g, 0).unwrap();
    println!("访问顺序: {:?}", order);
}
```

### 图构建

```rust
use rust_igraph::{Graph, GraphBuilder};

// 流式构建器
let g = GraphBuilder::undirected()
    .vertices(5)
    .edges(&[(0,1), (1,2), (2,3), (3,4)])
    .cycle(&[0, 1, 2, 3, 4])
    .build()
    .unwrap();

// 从边列表构建（自动推断顶点数）
let g = Graph::from_edges(&[(0,1), (1,2), (2,0)], false, None).unwrap();

// 通过 TryFrom 从切片构建
let g = Graph::try_from(vec![(0u32, 1), (1, 2), (2, 0)].as_slice()).unwrap();

// 从字符串构建（方便测试）
let g = Graph::from_edge_list_str("0 1\n1 2\n2 0").unwrap();

// 经典图生成器
let k5 = rust_igraph::full_graph(5, false, false).unwrap();
let ring = rust_igraph::cycle_graph(10, false, false).unwrap();
```

### 图代数（运算符重载）

```rust
use rust_igraph::Graph;

let a = Graph::from_edges(&[(0,1), (1,2)], false, None).unwrap();
let b = Graph::from_edges(&[(1,2), (2,0)], false, None).unwrap();

let union = &a | &b;          // 并集：任一图中的边
let intersection = &a & &b;   // 交集：两图共有的边
let difference = &a - &b;     // 差集：a 中有但 b 中没有的边
let complement = !&a;          // 补图：所有缺失的边
let disjoint = &a + &b;       // 不相交并（6 个顶点）
```

### 社区发现

```rust
use rust_igraph::{Graph, louvain};

let mut g = Graph::with_vertices(10);
// ... 添加构成两个簇的边 ...
let result = louvain(&g).unwrap();
println!("社区: {:?}", result.membership);
println!("模块度: {:.4}", result.modularity);
```

### 中心性分析

```rust
use rust_igraph::{Graph, pagerank, betweenness, katz_centrality};

let g = Graph::from_edges(
    &[(0,1), (1,2), (2,3), (3,4)], false, None
).unwrap();

let pr = pagerank(&g).unwrap();
let bc = betweenness(&g).unwrap();
let katz = katz_centrality(&g, 0.1, 1.0, None, None).unwrap();
println!("PageRank: {:?}", pr);
println!("Betweenness: {:?}", bc);
println!("Katz: {:?}", katz);
```

### 方法风格 API

最常用的操作可直接在 `Graph` 上调用：

```rust
use rust_igraph::Graph;

let g = Graph::from_edges(
    &[(0,1), (1,2), (2,0), (2,3), (3,4), (4,5), (5,3)],
    false, None
).unwrap();

// 结构查询
assert!(g.is_connected().unwrap());
println!("直径: {:?}", g.diameter().unwrap());
println!("围长: {:?}", g.girth().unwrap());
println!("三角形数: {}", g.count_triangles().unwrap());

// 中心性
let pr = g.pagerank().unwrap();
let bc = g.betweenness().unwrap();
let hc = g.harmonic_centrality().unwrap();

// 社区发现
let communities = g.louvain().unwrap();
println!("模块度: {:.4}", communities.modularity);

// 图生成
let er = Graph::erdos_renyi(1000, 0.01, 42).unwrap();
let ws = Graph::watts_strogatz(1000, 6, 0.1, 42).unwrap();
let ba = Graph::barabasi_albert(1000, 3, 42).unwrap();
```

## 性能

所有算法均用地道的 Rust 实现，精心优化缓存局部性和分配模式。每个主要算法都包含基准测试（via `criterion`）：

```bash
cargo bench --bench bench_louvain     # 社区发现
cargo bench --bench bench_bfs         # 遍历
cargo bench --bench bench_max_flow    # 网络流
cargo bench --bench bench_vf2         # 同构
```

## 项目状态

> **v0.7.0** — 已完成 315 个算法工作单元，1,297 个公开函数，
> 1,100 个测试，1,850 个一致性验证。API 正在向 `v1.0.0` 稳定推进。

| 类别 | 状态 |
|------|------|
| 核心数据结构 | 稳定 |
| 遍历（BFS、DFS） | 稳定 |
| 最短路径 | 稳定 |
| 中心性 | 稳定 |
| 社区发现 | 稳定 |
| 连通性 | 稳定 |
| 网络流 | 稳定 |
| 同构 | 稳定 |
| 图生成器 | 稳定 |
| 布局算法 | 稳定（16 种引擎） |
| I/O 格式 | 稳定（15 种函数） |
| 空间算法 | 稳定 |

## 发布

本仓库产出两个独立的发布产物：

| 包 | 注册中心 | 标签模式 | 触发条件 |
|----|----------|----------|----------|
| `rust-igraph` | [crates.io](https://crates.io/crates/rust-igraph) | `v*`（如 `v0.6.0`） | Rust 库发版 |
| `@graphrs/igraph-wasm` | [npm](https://www.npmjs.com/package/@graphrs/igraph-wasm) | `wasm-v*`（如 `wasm-v0.1.4`） | WASM 二进制发版 |

两者遵循独立的版本节奏。WASM 通过 [npm Trusted Publishing (OIDC)](https://docs.npmjs.com/trusted-publishers)
发布——零静态令牌，带密码学签名的溯源证明。

## 文档

- **[教程与指南](https://Totoro-jam.github.io/rust-igraph/book/)** — mdBook，包含入门、实践手册和架构概览
- **[API 参考](https://Totoro-jam.github.io/rust-igraph/rust_igraph/)** — 全部 1,297 个公开项的 rustdoc
- **[docs.rs](https://docs.rs/rust-igraph)** — 每次 crates.io 发版后自动发布

## 开发

```bash
cargo build                          # 构建
cargo test                           # 快速测试（8,494 个测试）
cargo test --all-features            # 完整测试（含 oracle + proptests）
cargo clippy -- -D warnings          # lint 检查
cargo doc --no-deps --open           # 本地浏览 API 文档
```

每个算法遵循 9 步 **AWU**（Algorithm Work Unit）流程，跟踪于
[`.codefuse/tracking/ALGORITHMS.md`](.codefuse/tracking/ALGORITHMS.md)。工程计划详见
[`docs/plans/MASTER_PLAN.md`](docs/plans/MASTER_PLAN.md)。

## 贡献

欢迎贡献！请参阅 [CONTRIBUTING.md](CONTRIBUTING.md) 了解指南。

无论是修复 bug、添加算法、改进文档还是优化性能——我们都非常欢迎。较大的改动请先开 issue 讨论。

## 许可证

GPL-2.0-or-later。与上游 igraph C 相同，允许直接参考翻译 C 源码。详见 [LICENSE](LICENSE)。

## 致谢

- [igraph](https://github.com/igraph/igraph) — igraph 团队（C 核心）
- [python-igraph](https://github.com/igraph/python-igraph) 和 [rigraph](https://github.com/igraph/rigraph)（参考测试套件）
